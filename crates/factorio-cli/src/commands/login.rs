use std::io::Write;

use clap::Parser;
use eyre::Result;
use factorio_mod_api::ModPortalClient;

use crate::App;

/// Log in to the mod portal API and store the obtained login token.
#[derive(Debug, Parser)]
pub struct LoginCommand;

impl LoginCommand {
    pub async fn execute(&self, app: &App) -> Result<()> {
        print!("User name or email: ");
        std::io::stdout().flush()?;
        let mut user_name = String::new();
        std::io::stdin().read_line(&mut user_name)?;
        let user_name = user_name.trim();

        let password = rpassword::prompt_password("Password: ")?;

        let token_path = app.api_token_path();
        std::fs::create_dir_all(
            token_path.parent().expect("token path should have a valid parent directory"),
        )?;

        let api_token = ModPortalClient::new()?.login(user_name, &password).await?;
        std::fs::write(token_path, serde_json::to_string(&api_token)?)?;

        Ok(())
    }
}
