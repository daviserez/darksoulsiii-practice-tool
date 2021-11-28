use std::str::FromStr;

use log::LevelFilter;
use serde::Deserialize;

use crate::memedit::*;
use crate::pointers::PointerChains;
use crate::util;
use crate::util::KeyState;

use super::flag::Flag;
use super::Command;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) settings: Settings,
    commands: Vec<CfgCommand>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Settings {
    pub(crate) log_level: LevelFilterSerde,
    pub(crate) display: KeyState,
    pub(crate) down: KeyState,
    pub(crate) up: KeyState,
    pub(crate) left: KeyState,
    pub(crate) right: KeyState,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd")]
enum CfgCommand {
    #[serde(rename = "savefile_manager")]
    SavefileManager {
        hotkey: KeyState,
    },
    #[serde(rename = "flag")]
    Flag {
        flag: FlagSpec,
        hotkey: KeyState,
    },
    #[serde(rename = "position")]
    Position {
        hotkey: KeyState,
    },
    #[serde(rename = "speed")]
    CycleSpeed {
        cycle_values: Vec<f32>,
        hotkey: KeyState,
    },
    #[serde(rename = "souls")]
    Souls {
        amount: u32,
        hotkey: KeyState,
    },
    #[serde(rename = "quitout")]
    Quitout {
        hotkey: KeyState,
    },
}

#[derive(Deserialize, Debug)]
#[serde(try_from = "String")]
pub(crate) struct LevelFilterSerde(log::LevelFilter);

impl LevelFilterSerde {
    pub(crate) fn inner(&self) -> log::LevelFilter {
        self.0
    }
}

impl TryFrom<String> for LevelFilterSerde {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(LevelFilterSerde(
            log::LevelFilter::from_str(&value)
                .map_err(|e| format!("Couldn't parse log level filter: {}", e))?,
        ))
    }
}

impl Config {
    pub(crate) fn parse(cfg: &str) -> Result<Self, String> {
        println!("{}", cfg);
        toml::from_str(cfg).map_err(|e| format!("TOML configuration parse error: {}", e))?
    }

    pub(crate) fn make_commands(&self, chains: &PointerChains) -> Vec<Box<dyn Command>> {
        self.commands
            .iter()
            .filter_map(|cmd| {
                if let CfgCommand::Flag { flag, hotkey } = cmd {
                    Some(
                        Box::new(Flag::new((flag.getter)(chains).clone(), hotkey.clone()))
                            as Box<dyn Command>,
                    )
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            settings: Settings {
                log_level: LevelFilterSerde(LevelFilter::Debug),
                display: KeyState::new(util::get_key_code("0").unwrap()),
                down: KeyState::new(util::get_key_code("down").unwrap()),
                up: KeyState::new(util::get_key_code("up").unwrap()),
                left: KeyState::new(util::get_key_code("left").unwrap()),
                right: KeyState::new(util::get_key_code("right").unwrap()),
            },
            commands: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
struct FlagSpec {
    label: String,
    getter: fn(&PointerChains) -> &Bitflag<u8>,
}

impl std::fmt::Debug for FlagSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlagSpec {{ label: {:?} }}", self.label)
    }
}

impl FlagSpec {
    fn new(label: &str, getter: fn(&PointerChains) -> &Bitflag<u8>) -> FlagSpec {
        FlagSpec {
            label: label.to_string(),
            getter,
        }
    }
}

impl TryFrom<String> for FlagSpec {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "all_no_damage" => Ok(FlagSpec::new("All no damage", |c| &c.all_no_damage)),
            "inf_stamina" => Ok(FlagSpec::new("Inf Stamina", |c| &c.inf_stamina)),
            "inf_focus" => Ok(FlagSpec::new("Inf Focus", |c| &c.inf_focus)),
            "inf_consumables" => Ok(FlagSpec::new("Inf Consumables", |c| &c.inf_consumables)),
            "deathcam" => Ok(FlagSpec::new("Deathcam", |c| &c.deathcam)),
            "no_death" => Ok(FlagSpec::new("No death", |c| &c.no_death)),
            "one_shot" => Ok(FlagSpec::new("One shot", |c| &c.one_shot)),
            "evt_draw" => Ok(FlagSpec::new("Event draw", |c| &c.evt_draw)),
            "evt_disable" => Ok(FlagSpec::new("Event disable", |c| &c.evt_disable)),
            "ai_disable" => Ok(FlagSpec::new("AI disable", |c| &c.ai_disable)),
            "rend_chr" => Ok(FlagSpec::new("Render characters", |c| &c.rend_chr)),
            "rend_obj" => Ok(FlagSpec::new("Render objects", |c| &c.rend_obj)),
            "rend_map" => Ok(FlagSpec::new("Render map", |c| &c.rend_map)),
            "rend_mesh_hi" => Ok(FlagSpec::new("Collision mesh (hi)", |c| &c.rend_mesh_hi)),
            "rend_mesh_lo" => Ok(FlagSpec::new("Collision mesh (lo)", |c| &c.rend_mesh_lo)),
            "all_draw_hit" => Ok(FlagSpec::new("All draw hit", |c| &c.all_draw_hit)),
            "ik_foot_ray" => Ok(FlagSpec::new("IK foot ray", |c| &c.ik_foot_ray)),
            "debug_sphere_1" => Ok(FlagSpec::new("Debug sphere 1", |c| &c.debug_sphere_1)),
            "debug_sphere_2" => Ok(FlagSpec::new("Debug sphere 2", |c| &c.debug_sphere_2)),
            "gravity" => Ok(FlagSpec::new("Gravity", |c| &c.gravity)),
            e => Err(format!("\"{}\" is not a valid flag specifier", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_parse() {
        println!(
            "{:#?}",
            toml::from_str::<toml::Value>(include_str!("../../../jdsd_dsiii_practice_tool.toml"))
        );
        println!(
            "{:#?}",
            Config::parse(include_str!("../../../jdsd_dsiii_practice_tool.toml"))
        );
    }

    // TODO tests with errors
}
