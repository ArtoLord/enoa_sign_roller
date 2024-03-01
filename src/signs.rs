use indoc::formatdoc;
use serde::Deserialize;

use crate::db::SignInfo;
use anyhow::Result;
use std::{fs, sync::OnceLock};
use std::collections::HashMap;

static DATA: OnceLock<HashMap<String, SignData>> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
struct SignData {
    id: String,
    name: String,
    difficulty: u32,
    description: String,
    effect: String,
    success_effect: String,
    failure_effect: String
}

pub fn load_signs(file_path: String) -> Result<()> {
    let data = fs::read_to_string(file_path)?;
    let data: Vec<SignData> = serde_json::from_str(&data)?;

    let mut values = HashMap::new();

    for s in data {
        values.insert(s.id.clone(), s);
    }

    let res = DATA.set(values);

    if res.is_err() {
        return Err(anyhow::anyhow!("Cannot load data from file"));
    }
    Ok(())
}

pub fn render_sign(sign: SignInfo) -> String {
    let sign = DATA.get().unwrap().get(&sign.id).unwrap();

    formatdoc!(r#"
    __**{}**__
    **Кости:** {}
    **Сложность:** {}

    > *{}*

    **Эффект:** {}
    **Успех:** {}
    **Провал:** {}
    "#, sign.name, sign.id, sign.difficulty, sign.description, sign.effect, sign.success_effect, sign.failure_effect)
}