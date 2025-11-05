use crate::memory::Memory;
use serde::{Deserialize, Serialize};

/// メモリーのレア度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryRarity {
    Common,      // 0.0-0.4
    Uncommon,    // 0.4-0.6
    Rare,        // 0.6-0.8
    Epic,        // 0.8-0.9
    Legendary,   // 0.9-1.0
}

impl MemoryRarity {
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 0.9 => MemoryRarity::Legendary,
            s if s >= 0.8 => MemoryRarity::Epic,
            s if s >= 0.6 => MemoryRarity::Rare,
            s if s >= 0.4 => MemoryRarity::Uncommon,
            _ => MemoryRarity::Common,
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            MemoryRarity::Common => "⚪",
            MemoryRarity::Uncommon => "🟢",
            MemoryRarity::Rare => "🔵",
            MemoryRarity::Epic => "🟣",
            MemoryRarity::Legendary => "🟡",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            MemoryRarity::Common => "COMMON",
            MemoryRarity::Uncommon => "UNCOMMON",
            MemoryRarity::Rare => "RARE",
            MemoryRarity::Epic => "EPIC",
            MemoryRarity::Legendary => "LEGENDARY",
        }
    }

    pub fn xp_value(&self) -> u32 {
        match self {
            MemoryRarity::Common => 100,
            MemoryRarity::Uncommon => 250,
            MemoryRarity::Rare => 500,
            MemoryRarity::Epic => 850,
            MemoryRarity::Legendary => 1000,
        }
    }
}

/// 診断タイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosisType {
    Innovator,      // 革新者（創造性高、実用性高）
    Philosopher,    // 哲学者（感情高、新規性高）
    Pragmatist,     // 実務家（実用性高、関連性高）
    Visionary,      // 夢想家（新規性高、感情高）
    Analyst,        // 分析家（全て平均的）
}

impl DiagnosisType {
    /// スコアから診断タイプを推定（公開用）
    pub fn from_memory(memory: &crate::memory::Memory) -> Self {
        // スコア内訳を推定
        let emotional = (memory.priority_score * 0.25).min(0.25);
        let relevance = (memory.priority_score * 0.25).min(0.25);
        let novelty = (memory.priority_score * 0.25).min(0.25);
        let utility = memory.priority_score - emotional - relevance - novelty;

        Self::from_score_breakdown(emotional, relevance, novelty, utility)
    }

    pub fn from_score_breakdown(
        emotional: f32,
        relevance: f32,
        novelty: f32,
        utility: f32,
    ) -> Self {
        if utility > 0.2 && novelty > 0.2 {
            DiagnosisType::Innovator
        } else if emotional > 0.2 && novelty > 0.2 {
            DiagnosisType::Philosopher
        } else if utility > 0.2 && relevance > 0.2 {
            DiagnosisType::Pragmatist
        } else if novelty > 0.2 && emotional > 0.18 {
            DiagnosisType::Visionary
        } else {
            DiagnosisType::Analyst
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            DiagnosisType::Innovator => "💡",
            DiagnosisType::Philosopher => "🧠",
            DiagnosisType::Pragmatist => "🎯",
            DiagnosisType::Visionary => "✨",
            DiagnosisType::Analyst => "📊",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            DiagnosisType::Innovator => "革新者",
            DiagnosisType::Philosopher => "哲学者",
            DiagnosisType::Pragmatist => "実務家",
            DiagnosisType::Visionary => "夢想家",
            DiagnosisType::Analyst => "分析家",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            DiagnosisType::Innovator => "創造的で実用的なアイデアを生み出す。常に新しい可能性を探求し、それを現実のものにする力を持つ。",
            DiagnosisType::Philosopher => "深い思考と感情を大切にする。抽象的な概念や人生の意味について考えることを好む。",
            DiagnosisType::Pragmatist => "現実的で効率的。具体的な問題解決に優れ、確実に結果を出す。",
            DiagnosisType::Visionary => "大胆な夢と理想を追い求める。常識にとらわれず、未来の可能性を信じる。",
            DiagnosisType::Analyst => "バランスの取れた思考。多角的な視点から物事を分析し、冷静に判断する。",
        }
    }
}

/// ゲーム風の結果フォーマッター
pub struct GameFormatter;

impl GameFormatter {
    /// メモリー作成結果をゲーム風に表示
    pub fn format_memory_result(memory: &Memory) -> String {
        let rarity = MemoryRarity::from_score(memory.priority_score);
        let xp = rarity.xp_value();
        let score_percentage = (memory.priority_score * 100.0) as u32;

        // スコア内訳を推定（各項目最大0.25として）
        let emotional = (memory.priority_score * 0.25).min(0.25);
        let relevance = (memory.priority_score * 0.25).min(0.25);
        let novelty = (memory.priority_score * 0.25).min(0.25);
        let utility = memory.priority_score - emotional - relevance - novelty;

        let diagnosis = DiagnosisType::from_score_breakdown(
            emotional,
            relevance,
            novelty,
            utility,
        );

        format!(
            r#"
╔══════════════════════════════════════════════════════════════╗
║                    🎲 メモリースコア判定                    ║
╚══════════════════════════════════════════════════════════════╝

⚡ 分析完了！ あなたの思考が記録されました

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 総合スコア
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   {} {} {}点
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🎯 詳細分析
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
💓 感情的インパクト:  {}
🔗 ユーザー関連性:    {}
✨ 新規性・独自性:    {}
⚙️  実用性:           {}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🎊 あなたのタイプ
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{} 【{}】

{}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🏆 報酬
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
💎 XP獲得: +{} XP
🎁 レア度: {} {}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

💬 AI の解釈
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📤 この結果をシェアしよう！
#aigpt #メモリースコア #{}
"#,
            rarity.emoji(),
            rarity.name(),
            score_percentage,
            Self::format_bar(emotional, 0.25),
            Self::format_bar(relevance, 0.25),
            Self::format_bar(novelty, 0.25),
            Self::format_bar(utility, 0.25),
            diagnosis.emoji(),
            diagnosis.name(),
            diagnosis.description(),
            xp,
            rarity.emoji(),
            rarity.name(),
            memory.interpreted_content,
            diagnosis.name(),
        )
    }

    /// シェア用の短縮テキストを生成
    pub fn format_shareable_text(memory: &Memory) -> String {
        let rarity = MemoryRarity::from_score(memory.priority_score);
        let score_percentage = (memory.priority_score * 100.0) as u32;
        let emotional = (memory.priority_score * 0.25).min(0.25);
        let relevance = (memory.priority_score * 0.25).min(0.25);
        let novelty = (memory.priority_score * 0.25).min(0.25);
        let utility = memory.priority_score - emotional - relevance - novelty;
        let diagnosis = DiagnosisType::from_score_breakdown(
            emotional,
            relevance,
            novelty,
            utility,
        );

        format!(
            r#"🎲 AIメモリースコア診断結果

{} {} {}点
{} 【{}】

{}

#aigpt #メモリースコア #AI診断"#,
            rarity.emoji(),
            rarity.name(),
            score_percentage,
            diagnosis.emoji(),
            diagnosis.name(),
            Self::truncate(&memory.content, 100),
        )
    }

    /// ランキング表示
    pub fn format_ranking(memories: &[&Memory], title: &str) -> String {
        let mut result = format!(
            r#"
╔══════════════════════════════════════════════════════════════╗
║                    🏆 {}                    ║
╚══════════════════════════════════════════════════════════════╝

"#,
            title
        );

        for (i, memory) in memories.iter().take(10).enumerate() {
            let rank_emoji = match i {
                0 => "🥇",
                1 => "🥈",
                2 => "🥉",
                _ => "　",
            };

            let rarity = MemoryRarity::from_score(memory.priority_score);
            let score = (memory.priority_score * 100.0) as u32;

            result.push_str(&format!(
                "{} {}位 {} {} {}点 - {}\n",
                rank_emoji,
                i + 1,
                rarity.emoji(),
                rarity.name(),
                score,
                Self::truncate(&memory.content, 40)
            ));
        }

        result.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        result
    }

    /// デイリーチャレンジ表示
    pub fn format_daily_challenge() -> String {
        // 今日の日付をシードにランダムなお題を生成
        let challenges = vec![
            "今日学んだことを記録しよう",
            "新しいアイデアを思いついた？",
            "感動したことを書き留めよう",
            "目標を一つ設定しよう",
            "誰かに感謝の気持ちを伝えよう",
        ];

        let today = chrono::Utc::now().ordinal();
        let challenge = challenges[today as usize % challenges.len()];

        format!(
            r#"
╔══════════════════════════════════════════════════════════════╗
║                  📅 今日のチャレンジ                         ║
╚══════════════════════════════════════════════════════════════╝

✨ {}

🎁 報酬: +200 XP
💎 完了すると特別なバッジが獲得できます！

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
"#,
            challenge
        )
    }

    /// プログレスバーを生成
    fn format_bar(value: f32, max: f32) -> String {
        let percentage = (value / max * 100.0) as u32;
        let filled = (percentage / 10) as usize;
        let empty = 10 - filled;

        format!(
            "[{}{}] {}%",
            "█".repeat(filled),
            "░".repeat(empty),
            percentage
        )
    }

    /// テキストを切り詰め
    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_rarity_from_score() {
        assert!(matches!(MemoryRarity::from_score(0.95), MemoryRarity::Legendary));
        assert!(matches!(MemoryRarity::from_score(0.85), MemoryRarity::Epic));
        assert!(matches!(MemoryRarity::from_score(0.7), MemoryRarity::Rare));
        assert!(matches!(MemoryRarity::from_score(0.5), MemoryRarity::Uncommon));
        assert!(matches!(MemoryRarity::from_score(0.3), MemoryRarity::Common));
    }

    #[test]
    fn test_diagnosis_type() {
        let diagnosis = DiagnosisType::from_score_breakdown(0.1, 0.1, 0.22, 0.22);
        assert!(matches!(diagnosis, DiagnosisType::Innovator));
    }

    #[test]
    fn test_format_bar() {
        let bar = GameFormatter::format_bar(0.15, 0.25);
        assert!(bar.contains("60%"));
    }
}
