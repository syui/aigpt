use crate::memory::Memory;
use crate::game_formatter::{MemoryRarity, DiagnosisType};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// コンパニオンキャラクター
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Companion {
    pub name: String,
    pub personality: CompanionPersonality,
    pub relationship_level: u32,        // レベル（経験値で上昇）
    pub affection_score: f32,           // 好感度 (0.0-1.0)
    pub trust_level: u32,               // 信頼度 (0-100)
    pub total_xp: u32,                  // 総XP
    pub last_interaction: DateTime<Utc>,
    pub shared_memories: Vec<String>,   // 共有された記憶のID
}

/// コンパニオンの性格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompanionPersonality {
    Energetic,   // 元気で冒険好き - 革新者と相性◎
    Intellectual, // 知的で思慮深い - 哲学者と相性◎
    Practical,   // 現実的で頼れる - 実務家と相性◎
    Dreamy,      // 夢見がちでロマンチック - 夢想家と相性◎
    Balanced,    // バランス型 - 分析家と相性◎
}

impl CompanionPersonality {
    pub fn emoji(&self) -> &str {
        match self {
            CompanionPersonality::Energetic => "⚡",
            CompanionPersonality::Intellectual => "📚",
            CompanionPersonality::Practical => "🎯",
            CompanionPersonality::Dreamy => "🌙",
            CompanionPersonality::Balanced => "⚖️",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            CompanionPersonality::Energetic => "元気で冒険好き",
            CompanionPersonality::Intellectual => "知的で思慮深い",
            CompanionPersonality::Practical => "現実的で頼れる",
            CompanionPersonality::Dreamy => "夢見がちでロマンチック",
            CompanionPersonality::Balanced => "バランス型",
        }
    }

    /// ユーザーの診断タイプとの相性
    pub fn compatibility(&self, user_type: &DiagnosisType) -> f32 {
        match (self, user_type) {
            (CompanionPersonality::Energetic, DiagnosisType::Innovator) => 0.95,
            (CompanionPersonality::Intellectual, DiagnosisType::Philosopher) => 0.95,
            (CompanionPersonality::Practical, DiagnosisType::Pragmatist) => 0.95,
            (CompanionPersonality::Dreamy, DiagnosisType::Visionary) => 0.95,
            (CompanionPersonality::Balanced, DiagnosisType::Analyst) => 0.95,
            // その他の組み合わせ
            _ => 0.7,
        }
    }
}

impl Companion {
    pub fn new(name: String, personality: CompanionPersonality) -> Self {
        Companion {
            name,
            personality,
            relationship_level: 1,
            affection_score: 0.0,
            trust_level: 0,
            total_xp: 0,
            last_interaction: Utc::now(),
            shared_memories: Vec::new(),
        }
    }

    /// 記憶を共有して反応を得る
    pub fn react_to_memory(&mut self, memory: &Memory, user_type: &DiagnosisType) -> CompanionReaction {
        let rarity = MemoryRarity::from_score(memory.priority_score);
        let xp = rarity.xp_value();

        // XPを加算
        self.total_xp += xp;

        // 好感度上昇（スコアと相性による）
        let compatibility = self.personality.compatibility(user_type);
        let affection_gain = memory.priority_score * compatibility * 0.1;
        self.affection_score = (self.affection_score + affection_gain).min(1.0);

        // 信頼度上昇（高スコアの記憶ほど上昇）
        if memory.priority_score >= 0.8 {
            self.trust_level = (self.trust_level + 5).min(100);
        }

        // レベルアップチェック
        let old_level = self.relationship_level;
        self.relationship_level = (self.total_xp / 1000) + 1;
        let level_up = self.relationship_level > old_level;

        // 記憶を共有リストに追加
        if memory.priority_score >= 0.6 {
            self.shared_memories.push(memory.id.clone());
        }

        self.last_interaction = Utc::now();

        // 反応メッセージを生成
        let message = self.generate_reaction_message(memory, &rarity, user_type);

        CompanionReaction {
            message,
            affection_gained: affection_gain,
            xp_gained: xp,
            level_up,
            new_level: self.relationship_level,
            current_affection: self.affection_score,
            special_event: self.check_special_event(),
        }
    }

    /// 記憶に基づく反応メッセージを生成
    fn generate_reaction_message(&self, memory: &Memory, rarity: &MemoryRarity, user_type: &DiagnosisType) -> String {
        let content_preview = if memory.content.len() > 50 {
            format!("{}...", &memory.content[..50])
        } else {
            memory.content.clone()
        };

        match (rarity, &self.personality) {
            // LEGENDARY反応
            (MemoryRarity::Legendary, CompanionPersonality::Energetic) => {
                format!(
                    "すごい！「{}」って本当に素晴らしいアイデアだね！\n\
                    一緒に実現させよう！ワクワクするよ！",
                    content_preview
                )
            }
            (MemoryRarity::Legendary, CompanionPersonality::Intellectual) => {
                format!(
                    "「{}」という考え、とても興味深いわ。\n\
                    深い洞察力を感じるの。もっと詳しく聞かせて？",
                    content_preview
                )
            }
            (MemoryRarity::Legendary, CompanionPersonality::Practical) => {
                format!(
                    "「{}」か。実現可能性が高そうだね。\n\
                    具体的な計画を一緒に立てようよ。",
                    content_preview
                )
            }
            (MemoryRarity::Legendary, CompanionPersonality::Dreamy) => {
                format!(
                    "「{}」...素敵♪ まるで夢みたい。\n\
                    あなたの想像力、本当に好きよ。",
                    content_preview
                )
            }

            // EPIC反応
            (MemoryRarity::Epic, _) => {
                format!(
                    "おお、「{}」って面白いね！\n\
                    あなたのそういうところ、好きだな。",
                    content_preview
                )
            }

            // RARE反応
            (MemoryRarity::Rare, _) => {
                format!(
                    "「{}」か。なるほどね。\n\
                    そういう視点、参考になるよ。",
                    content_preview
                )
            }

            // 通常反応
            _ => {
                format!(
                    "「{}」について考えてるんだね。\n\
                    いつも色々考えてて尊敬するよ。",
                    content_preview
                )
            }
        }
    }

    /// スペシャルイベントチェック
    fn check_special_event(&self) -> Option<SpecialEvent> {
        // 好感度MAXイベント
        if self.affection_score >= 1.0 {
            return Some(SpecialEvent::MaxAffection);
        }

        // レベル10到達
        if self.relationship_level == 10 {
            return Some(SpecialEvent::Level10);
        }

        // 信頼度MAX
        if self.trust_level >= 100 {
            return Some(SpecialEvent::MaxTrust);
        }

        None
    }

    /// デイリーメッセージを生成
    pub fn generate_daily_message(&self) -> String {
        let messages = match &self.personality {
            CompanionPersonality::Energetic => vec![
                "おはよう！今日は何か面白いことある？",
                "ねえねえ、今日は一緒に新しいことやろうよ！",
                "今日も元気出していこー！",
            ],
            CompanionPersonality::Intellectual => vec![
                "おはよう。今日はどんな発見があるかしら？",
                "最近読んだ本の話、聞かせてくれない？",
                "今日も一緒に学びましょう。",
            ],
            CompanionPersonality::Practical => vec![
                "おはよう。今日の予定は？",
                "やることリスト、一緒に確認しようか。",
                "今日も効率よくいこうね。",
            ],
            CompanionPersonality::Dreamy => vec![
                "おはよう...まだ夢の続き見てたの。",
                "今日はどんな素敵なことが起こるかな♪",
                "あなたと過ごす時間、大好き。",
            ],
            CompanionPersonality::Balanced => vec![
                "おはよう。今日も頑張ろうね。",
                "何か手伝えることある？",
                "今日も一緒にいられて嬉しいよ。",
            ],
        };

        let today = chrono::Utc::now().ordinal();
        messages[today as usize % messages.len()].to_string()
    }
}

/// コンパニオンの反応
#[derive(Debug, Serialize)]
pub struct CompanionReaction {
    pub message: String,
    pub affection_gained: f32,
    pub xp_gained: u32,
    pub level_up: bool,
    pub new_level: u32,
    pub current_affection: f32,
    pub special_event: Option<SpecialEvent>,
}

/// スペシャルイベント
#[derive(Debug, Serialize)]
pub enum SpecialEvent {
    MaxAffection,   // 好感度MAX
    Level10,        // レベル10到達
    MaxTrust,       // 信頼度MAX
    FirstDate,      // 初デート
    Confession,     // 告白
}

impl SpecialEvent {
    pub fn message(&self, companion_name: &str) -> String {
        match self {
            SpecialEvent::MaxAffection => {
                format!(
                    "💕 特別なイベント発生！\n\n\
                    {}:「ねえ...あのね。\n\
                    　　　いつも一緒にいてくれてありがとう。\n\
                    　　　あなたのこと、すごく大切に思ってるの。\n\
                    　　　これからも、ずっと一緒にいてね？」\n\n\
                    🎊 {} の好感度がMAXになりました！",
                    companion_name, companion_name
                )
            }
            SpecialEvent::Level10 => {
                format!(
                    "🎉 レベル10到達！\n\n\
                    {}:「ここまで一緒に来られたね。\n\
                    　　　あなたとなら、どこまでも行けそう。」",
                    companion_name
                )
            }
            SpecialEvent::MaxTrust => {
                format!(
                    "✨ 信頼度MAX！\n\n\
                    {}:「あなたのこと、心から信頼してる。\n\
                    　　　何でも話せるって、すごく嬉しいよ。」",
                    companion_name
                )
            }
            SpecialEvent::FirstDate => {
                format!(
                    "💐 初デートイベント！\n\n\
                    {}:「今度、二人でどこか行かない？」",
                    companion_name
                )
            }
            SpecialEvent::Confession => {
                format!(
                    "💝 告白イベント！\n\n\
                    {}:「好きです。付き合ってください。」",
                    companion_name
                )
            }
        }
    }
}

/// コンパニオンフォーマッター
pub struct CompanionFormatter;

impl CompanionFormatter {
    /// 反応を表示
    pub fn format_reaction(companion: &Companion, reaction: &CompanionReaction) -> String {
        let affection_bar = Self::format_affection_bar(reaction.current_affection);
        let level_up_text = if reaction.level_up {
            format!("\n🎊 レベルアップ！ Lv.{} → Lv.{}", reaction.new_level - 1, reaction.new_level)
        } else {
            String::new()
        };

        let special_event_text = if let Some(ref event) = reaction.special_event {
            format!("\n\n{}", event.message(&companion.name))
        } else {
            String::new()
        };

        format!(
            r#"
╔══════════════════════════════════════════════════════════════╗
║                💕 {} の反応                              ║
╚══════════════════════════════════════════════════════════════╝

{} {}:
「{}」

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
💕 好感度: {} (+{:.1}%)
💎 XP獲得: +{} XP{}
🏆 レベル: Lv.{}
🤝 信頼度: {} / 100
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}
"#,
            companion.name,
            companion.personality.emoji(),
            companion.name,
            reaction.message,
            affection_bar,
            reaction.affection_gained * 100.0,
            reaction.xp_gained,
            level_up_text,
            companion.relationship_level,
            companion.trust_level,
            special_event_text
        )
    }

    /// プロフィール表示
    pub fn format_profile(companion: &Companion) -> String {
        let affection_bar = Self::format_affection_bar(companion.affection_score);

        format!(
            r#"
╔══════════════════════════════════════════════════════════════╗
║                💕 {} のプロフィール                     ║
╚══════════════════════════════════════════════════════════════╝

{} 性格: {}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 ステータス
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🏆 関係レベル: Lv.{}
💕 好感度: {}
🤝 信頼度: {} / 100
💎 総XP: {} XP
📚 共有記憶: {}個
🕐 最終交流: {}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

💬 今日のひとこと:
「{}」
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
"#,
            companion.name,
            companion.personality.emoji(),
            companion.personality.name(),
            companion.relationship_level,
            affection_bar,
            companion.trust_level,
            companion.total_xp,
            companion.shared_memories.len(),
            companion.last_interaction.format("%Y-%m-%d %H:%M"),
            companion.generate_daily_message()
        )
    }

    fn format_affection_bar(affection: f32) -> String {
        let hearts = (affection * 10.0) as usize;
        let filled = "❤️".repeat(hearts);
        let empty = "🤍".repeat(10 - hearts);
        format!("{}{} {:.0}%", filled, empty, affection * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_companion_creation() {
        let companion = Companion::new(
            "エミリー".to_string(),
            CompanionPersonality::Energetic,
        );
        assert_eq!(companion.name, "エミリー");
        assert_eq!(companion.relationship_level, 1);
        assert_eq!(companion.affection_score, 0.0);
    }

    #[test]
    fn test_compatibility() {
        let personality = CompanionPersonality::Energetic;
        let innovator = DiagnosisType::Innovator;
        assert_eq!(personality.compatibility(&innovator), 0.95);
    }
}
