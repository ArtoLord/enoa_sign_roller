use crate::db::SignInfo;


pub fn render_sign(sign: SignInfo) -> String {
    format!("Твое знамение: {}", sign.id)
}