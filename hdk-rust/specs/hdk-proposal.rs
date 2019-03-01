#![hdk::zome] // this attribute-like macro operates on the whole module. This is equivalent to define_zome! in the old HDK

#[hdk:entry] // instructs the top level macro that a entry should be created that maps to this struct
struct Post {
    content: String,
}

#[hdk:entry(validation = "validate_blog", validation_package = ChainFull)]
// provides custom validation callback and also an idea of how links could be defined
struct Blog {
    owner: Agent,
    #[hdk::links("posts")]
    posts: vec<Post>,
}

fn validate_blog(entry: BlogEntry, validation_data: ValidationData) -> ZomeApiResult<()> {

}

/////////////////////////////////////////////////////////////////

#[hdk::genesis] // genesis callback
fn genesis() -> bool {
	true
}

#[hdk::receive] // receive callback
fn receive() -> ZomeApiResult<()> {
}

// a zome api function that belongs to these two traits 
// (this maybe needs some discussion)
#[hdk::zome_function(traits = ["zome", "bridge"])]
fn get_posts() -> ZomeApiResult<String> {
}

#[hdk::zome_function(traits = ["plumbus"])]
fn delete_all_posts_idc() -> ZomeApiResult<> {
}