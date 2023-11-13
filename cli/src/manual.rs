use clap_mangen::Man;
use std::fs::{create_dir_all, File};
use std::path::Path;
use std::sync::Arc;

pub fn generate_manual(cmd: clap::Command, out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    create_dir_all(out)?;

    Man::new(cmd.clone()).render(&mut File::create(out.join("typst-ts-cli.1")).unwrap())?;

    let mut borrow_str = vec![];

    for subcmd in cmd.get_subcommands() {
        let name: Arc<str> = format!("typst-ts-cli-{}", subcmd.get_name()).into();
        Man::new(
            subcmd
                .clone()
                .name(unsafe { std::mem::transmute::<&str, &'static str>(name.as_ref()) }),
        )
        .render(&mut File::create(out.join(format!("{name}.1")))?)?;
        borrow_str.push(name);
    }

    Ok(())
}
