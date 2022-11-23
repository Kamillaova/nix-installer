use crate::action::base::{CreateFile, CreateFileError};
use crate::{
    action::{Action, ActionDescription, ActionImplementation, ActionState},
    BoxableError,
};
use reqwest::Url;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct PlaceChannelConfiguration {
    channels: Vec<(String, Url)>,
    create_file: CreateFile,
    action_state: ActionState,
}

impl PlaceChannelConfiguration {
    #[tracing::instrument(skip_all)]
    pub async fn plan(
        channels: Vec<(String, Url)>,
        force: bool,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let buf = channels
            .iter()
            .map(|(name, url)| format!("{} {}", url, name))
            .collect::<Vec<_>>()
            .join("\n");
        let create_file = CreateFile::plan(
            dirs::home_dir()
                .ok_or_else(|| PlaceChannelConfigurationError::NoRootHome.boxed())?
                .join(".nix-channels"),
            None,
            None,
            0o0664,
            buf,
            force,
        )
        .await?;
        Ok(Self {
            create_file,
            channels,
            action_state: ActionState::Uncompleted,
        })
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "place_channel_configuration")]
impl Action for PlaceChannelConfiguration {
    fn tracing_synopsis(&self) -> String {
        format!(
            "Place channel configuration at `{}`",
            self.create_file.path.display()
        )
    }

    fn execute_description(&self) -> Vec<ActionDescription> {
        vec![ActionDescription::new(self.tracing_synopsis(), vec![])]
    }

    #[tracing::instrument(skip_all, fields(
        channels = self.channels.iter().map(|(c, u)| format!("{c}={u}")).collect::<Vec<_>>().join(", "),
    ))]
    async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let Self {
            create_file,
            channels: _,
            action_state: _,
        } = self;

        create_file.try_execute().await?;

        Ok(())
    }

    fn revert_description(&self) -> Vec<ActionDescription> {
        vec![ActionDescription::new(
            format!(
                "Remove channel configuration at `{}`",
                self.create_file.path.display()
            ),
            vec![],
        )]
    }

    #[tracing::instrument(skip_all, fields(
        channels = self.channels.iter().map(|(c, u)| format!("{c}={u}")).collect::<Vec<_>>().join(", "),
    ))]
    async fn revert(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let Self {
            create_file,
            channels: _,
            action_state: _,
        } = self;

        create_file.revert().await?;

        Ok(())
    }

    fn action_state(&self) -> ActionState {
        self.action_state
    }

    fn set_action_state(&mut self, action_state: ActionState) {
        self.action_state = action_state;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlaceChannelConfigurationError {
    #[error("Creating file")]
    CreateFile(
        #[source]
        #[from]
        CreateFileError,
    ),
    #[error("No root home found to place channel configuration in")]
    NoRootHome,
}