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
    difficulty: i32,
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
    let sign_desc = DATA.get().unwrap().get(&sign.id).unwrap();

    let mut res = formatdoc!(r#"
    __**{}**__
    **Кости:** {}
    **Сложность:** {}

    > *{}*

    **Эффект:** {}
    "#, sign_desc.name, sign_desc.id, sign_desc.difficulty, sign_desc.description, sign_desc.effect);

    match sign.state {
        crate::db::SignState::Created => res.push_str(&formatdoc!(r#"
            **Успех:** {}
            **Провал:** {}
            "#, sign_desc.success_effect, sign_desc.failure_effect)
        ),
        crate::db::SignState::Success { by_user_id: _ } => res.push_str(&formatdoc!(r#"
            **Эффект после изменения:** {}
            "#, sign_desc.success_effect)
        ),
        crate::db::SignState::Failed { by_user_id: _ } => res.push_str(&formatdoc!(r#"
            **Эффект после изменения:** {}
            "#, sign_desc.failure_effect)
        ),
    };

    res
}

pub fn get_difficulty(sign_id: String) -> i32 {
    let sign = DATA.get().unwrap().get(&sign_id).unwrap();

    sign.difficulty
}