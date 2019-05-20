use crate::types::{EffectAbstract, EffectGroup};
use holochain_core::{
    action::{Action, CommitKey},
    network::entry_with_header::EntryWithHeader,
};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    entry::Entry,
};
use std::{collections::HashMap, sync::Arc};

fn effect1<P: 'static + Send + Sync + Fn(&Action) -> bool>(
    description: String,
    group: EffectGroup,
    predicate: P,
) -> Vec<EffectAbstract> {
    vec![EffectAbstract {
        description,
        group,
        predicate: Arc::new(Box::new(predicate)),
    }]
}

pub struct CausalityModel {
    commit_cache: HashMap<Address, CommitKey>,
}

impl CausalityModel {
    pub fn new() -> Self {
        Self {
            commit_cache: HashMap::new(),
        }
    }

    pub fn resolve_action(&mut self, action: &Action) -> Vec<EffectAbstract> {
        match action {
            Action::Commit(data) => {
                let (entry, _, _) = data.clone();
                self.commit_cache.insert(entry.address(), data.clone());
                Vec::new()
            }
            Action::Publish(address) => {
                if let Some((committed_entry, maybe_crud_link, _provenances)) =
                    self.commit_cache.remove(&address)
                {
                    let committed_entry = committed_entry.clone();
                    let mut fx = Vec::new();

                    let mut crud_effects =
                        Self::reduce_crud(&committed_entry, maybe_crud_link.clone());

                    let mut publish_effects = effect1(
                        "Publish -> Hold".to_string(),
                        EffectGroup::Validators,
                        move |a| {
                            if let Action::Hold(EntryWithHeader { entry, header: _ }) = a.clone() {
                                entry == committed_entry.clone()
                            } else {
                                false
                            }
                        },
                    );

                    fx.append(&mut crud_effects);
                    fx.append(&mut publish_effects);
                    fx
                } else {
                    // TODO: use Result
                    panic!(
                        "Attempted to Publish entry before committing {}",
                        address.to_string()
                    );
                }
            }
            Action::AddPendingValidation(pending) => {
                let address = pending.entry_with_header.entry.address();
                let workflow = pending.workflow.clone();
                effect1(
                    "AddPendingValidation -> RemovePendingValidation".to_string(),
                    EffectGroup::Owner,
                    move |a| {
                        //println!("WAITER: Action::AddPendingValidation -> Action::RemovePendingValidation");
                        *a == Action::RemovePendingValidation((address.clone(), workflow.clone()))
                    },
                )
            }
            _ => Vec::new(),
        }
    }

    fn reduce_crud(
        committed_entry: &Entry,
        maybe_crud_link: Option<Address>,
    ) -> Vec<EffectAbstract> {
        let target = match committed_entry.clone() {
            Entry::App(_, _) => maybe_crud_link.clone().map(|crud_link_address| {
                Action::UpdateEntry((crud_link_address.clone(), committed_entry.address()))
            }),
            Entry::Deletion(deletion_entry) => Some(Action::RemoveEntry((
                deletion_entry.clone().deleted_entry_address(),
                committed_entry.address(),
            ))),
            Entry::LinkAdd(link_add_entry) => Some(Action::AddLink(link_add_entry.link().clone())),
            Entry::LinkRemove(link_add_entry) => {
                Some(Action::AddLink(link_add_entry.link().clone()))
            }
            _ => None,
        };

        target
            .map(|t| {
                effect1("CRUD".to_string(), EffectGroup::Validators, move |a| {
                    *a == t
                })
            })
            .unwrap_or(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::{Action::*, *};
    use holochain_core_types::{chain_header::test_chain_header, json::JsonString};
    use std::panic;

    fn mk_entry(ty: &'static str, content: &'static str) -> Entry {
        Entry::App(ty.into(), JsonString::from_json(content))
    }

    fn mk_entry_wh(entry: Entry) -> EntryWithHeader {
        EntryWithHeader {
            entry,
            header: test_chain_header(),
        }
    }

    #[test]
    fn commit_grows_cache() {
        let mut model = CausalityModel::new();
        let entry = mk_entry("t1", "x");
        let commit_key = (entry.clone(), None, Vec::new());

        model.resolve_action(&Commit(commit_key.clone()));
        assert_eq!(model.commit_cache.get(&entry.address()), Some(&commit_key));
    }

    #[test]
    fn publish_shrinks_cache() {
        let mut model = CausalityModel::new();
        let entry = mk_entry("t1", "x");
        let commit_key = (entry.clone(), None, Vec::new());

        model.resolve_action(&Commit(commit_key.clone()));
        model.resolve_action(&Publish(entry.address()));
        assert!(model.commit_cache.is_empty());
    }

    #[test]
    fn publish_fx() {
        let mut model = CausalityModel::new();
        let entry = mk_entry("t1", "x");
        let commit_key = (entry.clone(), None, Vec::new());
        let entry_wh = mk_entry_wh(entry.clone());

        model.resolve_action(&Commit(commit_key.clone()));
        let fx = model.resolve_action(&Publish(entry.address()));
        assert_eq!(fx.len(), 1);
        assert_eq!(fx[0].group, EffectGroup::Validators);
        assert_eq!(
            fx.into_iter()
                .map(|eff| (eff.predicate)(&Hold(entry_wh.clone())))
                .collect::<Vec<_>>(),
            vec![true]
        );
    }

    #[test]
    fn publish_panics_without_matching_commit() {
        let mut model = CausalityModel::new();
        let entry_1 = mk_entry("t1", "x");
        let entry_2 = mk_entry("t1", "y");
        let commit_key = (entry_1.clone(), None, Vec::new());

        model.resolve_action(&Commit(commit_key.clone()));
        let r = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            model.resolve_action(&Publish(entry_2.address()))
        }));
        assert!(r.is_err());
    }

}
