# Validate Agent

In Holochain the validation of the second entry on the chain, the AgentID, plays a important role because it serves as place to define membrane functions.  Therefore we have created a special validation function that must always be defined in every zome.

Example:

``` rust
validate_agent: |validation_data : EntryValidationData::<AgentId>| {{
    if let EntryValidationData::Create{entry, ..} = validation_data {
        let agent = entry as AgentId;
        if agent.nick == "reject_agent::app" {
            Err("This agent will always be rejected".into())
        } else {
            Ok(())
        }
    } else {
        Err("Cannot update or delete an agent at this time".into())
    }
}}
```
