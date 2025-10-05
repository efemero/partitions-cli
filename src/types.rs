// types.rs
//
// Copyright (c) 2025 François Bastien
// All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//
// This program is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
// Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program. If not, see <https:www.gnu.org/licenses/>.

use std::str::FromStr;

use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString};

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    strum::Display,
    EnumIter,
    EnumString,
    Hash,
    PartialEq,
    Eq,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    Baryton,
    Basse,
    Bugle,
    CaisseClaire,
    Clarinette,
    Contrebasse,
    Cor,
    Euphonium,
    Flute,
    GrosseCaisse,
    Piccolo,
    SaxophoneAlto,
    SaxophoneBaryton,
    SaxophoneSoprano,
    SaxophoneTenor,
    Trombone,
    Trompette,
    Tuba,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    strum::Display,
    EnumIter,
    EnumString,
    Hash,
    PartialEq,
    Eq,
)]
#[allow(clippy::upper_case_acronyms)]
pub enum Voice {
    I,
    II,
    III,
    Solo,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    strum::Display,
    EnumIter,
    EnumString,
    Hash,
    PartialEq,
    Eq,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Clef {
    ClefFa,
    ClefSol,
    ClefDrums,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    strum::Display,
    EnumIter,
    EnumString,
    Hash,
    PartialEq,
    Eq,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    Ut,
    Mib,
    Fa,
    Sib,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    strum::Display,
    EnumIter,
    EnumString,
    Hash,
    PartialEq,
    Eq,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Format {
    A4,
    Carnet,
}
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    strum::Display,
    EnumIter,
    EnumString,
    Hash,
    PartialEq,
    Eq,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Concerts,
    Animations,
    Marches,
}
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct MusicSheet {
    pub source: Utf8PathBuf,
    pub pdf: Utf8PathBuf,
    pub instrument: Instrument,
    pub tone: Option<Tone>,
    pub clef: Clef,
    pub voice: Option<Voice>,
    pub title: String,
    pub format: Format,
    pub category: Category,
}

impl TryFrom<Utf8PathBuf> for MusicSheet {
    type Error = anyhow::Error;
    fn try_from(source: Utf8PathBuf) -> Result<Self> {
        let mut components = source.components().rev();
        let filename = components
            .next()
            .ok_or(anyhow!("Empty or missing filename"))?
            .as_os_str()
            .to_string_lossy()
            .strip_suffix(".ly")
            .ok_or(anyhow!("Cannot strip suffix '.ly'"))?
            .to_owned();
        let format = components
            .next()
            .ok_or(anyhow!("Empty or missing format"))?
            .as_os_str()
            .to_string_lossy()
            .into_owned();
        let title = components
            .next()
            .ok_or(anyhow!("Empty or missing format"))?
            .as_os_str()
            .to_string_lossy()
            .into_owned();
        let category = components
            .next()
            .ok_or(anyhow!("Empty or missing format"))?
            .as_os_str()
            .to_string_lossy()
            .into_owned();
        let format = Format::from_str(&format)?;
        let category = Category::from_str(&category)?;
        let mut file_split = filename.split("_");
        let i = file_split
            .next()
            .ok_or(anyhow!("Missing instrument name in filename"))?;
        let i = match i {
            "grosse" | "caisse" | "saxophone" => {
                let i2 = file_split
                    .next()
                    .ok_or(anyhow!("Missing instrument name in filename"))?;
                &format!("{i}_{i2}")
            }
            _ => i,
        };
        let instrument = Instrument::from_str(i)?;
        let clef;
        let voice;
        let tone;
        match instrument {
            Instrument::CaisseClaire | Instrument::GrosseCaisse => {
                clef = Clef::ClefDrums;
                voice = None;
                tone = None
            }
            _ => {
                let v = file_split
                    .next()
                    .ok_or(anyhow!("Missing voice name in filename"))?;
                let t = file_split
                    .next()
                    .ok_or(anyhow!("Missing tone name in filename"))?;
                let c = file_split.join("_");
                voice = Some(Voice::from_str(v)?);
                tone = Some(Tone::from_str(t)?);
                clef = Clef::from_str(&c).unwrap_or(Clef::ClefSol);
            }
        };
        let mut pdf: Utf8PathBuf = "pdf".into();
        pdf.push(format.to_string());
        let mut filename = format!("{title}_{instrument}");
        if let Some(voice) = voice {
            filename.push('_');
            filename.push_str(&voice.to_string());
        };
        if let Some(tone) = tone {
            filename.push('_');
            filename.push_str(&tone.to_string());
        };
        if clef == Clef::ClefFa {
            filename.push('_');
            filename.push_str(&clef.to_string());
        };
        pdf.push(filename);
        Ok(Self {
            pdf,
            source,
            instrument,
            clef,
            voice,
            tone,
            format,
            title,
            category,
        })
    }
}
