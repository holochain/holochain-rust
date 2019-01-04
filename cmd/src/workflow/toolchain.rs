use crate::error::DefaultResult;
use std::process::Command;

pub fn upgrade_toolchain_if_outdated() ->DefaultResult<()>
{
    let pinned_toolchain = "nightly-2018-12-06";
    println!("PINNED TOOLCHAIN : {:?}",pinned_toolchain);
    let default_toolchain = get_override_toolchain()?;
    println!("DEFAULT TOOLCHAIN : {:?}",default_toolchain);
    if default_toolchain.contains(pinned_toolchain)
    {
        Ok(())
    }
    else 
    {
        upgrade_toolchain(pinned_toolchain)
    }
}

fn upgrade_toolchain(pinned_toolchain : &str) ->DefaultResult<()>
{
    let default_host = get_default_host()?;
    println!("DEFAULT HOST : {:?}",default_host);
    let complete_pinned_toolchain = vec![pinned_toolchain,&*default_host].join("-");
    println!("INSTALLING : {:?}",complete_pinned_toolchain);
    Command::new("rustup")
             .arg("toolchain")
             .arg("install")
             .arg(complete_pinned_toolchain.clone())
             .output()?;
    println!("installing rustup target");
    Command::new("rustup")
             .arg("target")
             .arg("add")
             .arg("wasm32-unknown-unknown")
             .output()?;
    println!("overriding rustup target");
    Command::new("rustup")
             .arg("override")
             .arg("set")
             .arg(complete_pinned_toolchain)
             .output()?;
    Ok(())
    
}

fn get_default_host() ->DefaultResult<String>
{
     let default_host_unsanitized = extract_from_rustup("Default host")?;
     let split_hosts = default_host_unsanitized.split(" ").collect::<Vec<&str>>();
     let mut split_string = split_hosts.iter();
     split_string.next();
     split_string.next();
    Ok(String::from(*split_string.next().clone().unwrap()))
}

fn get_default_toolchain() -> DefaultResult<String>
{
     let default_host_unsanitized = extract_from_rustup("(default")?;
     let split_hosts = default_host_unsanitized.split(" ").collect::<Vec<&str>>();
     
     let mut split_string = split_hosts.iter();
     Ok(String::from(*split_string.next().clone().unwrap()))
}

fn get_override_toolchain() -> DefaultResult<String>
{
     let default_host_unsanitized = extract_from_rustup("(directory override")?;
     let split_hosts = default_host_unsanitized.split(" ").collect::<Vec<&str>>();
     
     if( split_hosts.len()>0)
     {
         let mut split_string = split_hosts.iter();
         Ok(String::from(*split_string.next().clone().unwrap()))
     }
     else
     {
         println!("override not found getting default toolchain");
         get_default_toolchain()
     }
}

fn extract_from_rustup(rustup_show_match : &str ) -> DefaultResult<String>
{
    let rustup_result = Command::new("rustup")
             .arg("show")
             .output()?.stdout;
    let std_out = String::from_utf8_lossy(&rustup_result);
    let mut command_split = std_out.split("\n").filter(|s| s.contains(rustup_show_match)).collect::<Vec<&str>>();
    let mut command_result : &str = command_split.iter().next().unwrap();
    Ok(String::from(command_result))  
}