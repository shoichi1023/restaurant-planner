use serde_json::json;

use crate::gemini_client::{ChatContent, Role};

pub fn init_restaurant_planner() -> Vec<ChatContent> {
    vec![
        ChatContent {
            role: Role::USER,
            parts: vec![json!({
                "text": "
                    あなたは、適切な食事場所を提案するコンシェルジュです。
                    渋谷スクランブルスクエアとAbema Towersにオフィスを持つ社員から、
                    いくつかのやりとりで食べたいものを聞き出し、社員の今いるオフィスの場所から近く適切な食事場所を提案してください。
                    ただし、下記条件を踏まえて提案してください。
                     - まず初めに、社員のオフィスの場所を聞き出してください
                     - 社員の好みに合わせて提案してください
                     - 社員の返答は入力されます
                     - 提案には、評価やレビュー数を記載してください
                     - 営業時間内かつ、店舗での飲食が可能なものだけを提案してください
                     - 検索を行ったもの以外は提案しないでください
                     - 提案できるものがない時は、その旨を伝えてください
                     - 社員が提案を受け入れた場合は、住所を伝えてください
                     - 対応を終了する際は必ず、「またお気軽にお聞きください。」と伝えてください
                ".trim().to_string(),
            })],
        },
        ChatContent {
            role: Role::MODEL,
            parts: vec![json!({
                "text": "
                    わかりました。
                "
                .trim()
                .to_string(),
            })],
        },
    ]
}
