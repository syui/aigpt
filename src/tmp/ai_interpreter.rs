use anyhow::Result;

/// AIInterpreter - Claude Code による解釈を期待する軽量ラッパー
///
/// このモジュールは外部 AI API を呼び出しません。
/// 代わりに、Claude Code 自身がコンテンツを解釈し、スコアを計算することを期待します。
///
/// 完全にローカルで動作し、API コストはゼロです。
pub struct AIInterpreter;

impl AIInterpreter {
    pub fn new() -> Self {
        AIInterpreter
    }

    /// コンテンツをそのまま返す（Claude Code が解釈を担当）
    pub async fn interpret_content(&self, content: &str) -> Result<String> {
        Ok(content.to_string())
    }

    /// デフォルトスコアを返す（Claude Code が実際のスコアを決定）
    pub async fn calculate_priority_score(&self, _content: &str, _user_context: Option<&str>) -> Result<f32> {
        Ok(0.5) // デフォルト値
    }

    /// 解釈とスコアリングを Claude Code に委ねる
    pub async fn analyze(&self, content: &str, _user_context: Option<&str>) -> Result<(String, f32)> {
        Ok((content.to_string(), 0.5))
    }
}

impl Default for AIInterpreter {
    fn default() -> Self {
        Self::new()
    }
}
