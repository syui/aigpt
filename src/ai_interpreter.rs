use anyhow::{Context, Result};

#[cfg(feature = "ai-analysis")]
use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    set_key,
};

pub struct AIInterpreter {
    #[cfg(feature = "ai-analysis")]
    api_key: Option<String>,
}

impl AIInterpreter {
    pub fn new() -> Self {
        #[cfg(feature = "ai-analysis")]
        {
            let api_key = std::env::var("OPENAI_API_KEY").ok();
            if let Some(ref key) = api_key {
                set_key(key.clone());
            }
            AIInterpreter { api_key }
        }
        #[cfg(not(feature = "ai-analysis"))]
        {
            AIInterpreter {}
        }
    }

    /// AI解釈: 元のコンテンツを解釈して新しい表現を生成
    #[cfg(feature = "ai-analysis")]
    pub async fn interpret_content(&self, content: &str) -> Result<String> {
        if self.api_key.is_none() {
            return Ok(content.to_string());
        }

        let messages = vec![
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::System,
                content: Some("あなたは記憶を解釈するAIです。与えられたテキストを解釈し、より深い意味や文脈を抽出してください。元のテキストの本質を保ちながら、新しい視点や洞察を加えてください。".to_string()),
                name: None,
                function_call: None,
            },
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                content: Some(format!("以下のテキストを解釈してください:\n\n{}", content)),
                name: None,
                function_call: None,
            },
        ];

        let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
            .create()
            .await
            .context("Failed to create chat completion")?;

        let response = chat_completion
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_else(|| content.to_string());

        Ok(response)
    }

    #[cfg(not(feature = "ai-analysis"))]
    pub async fn interpret_content(&self, content: &str) -> Result<String> {
        Ok(content.to_string())
    }

    /// 心理判定: テキストの重要度を0.0-1.0のスコアで評価
    #[cfg(feature = "ai-analysis")]
    pub async fn calculate_priority_score(&self, content: &str, user_context: Option<&str>) -> Result<f32> {
        if self.api_key.is_none() {
            return Ok(0.5); // デフォルトスコア
        }

        let context_info = user_context
            .map(|ctx| format!("\n\nユーザーコンテキスト: {}", ctx))
            .unwrap_or_default();

        let messages = vec![
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::System,
                content: Some(format!(
                    "あなたは記憶の重要度を評価するAIです。以下の基準で0.0-1.0のスコアをつけてください:\n\
                    - 感情的インパクト (0.0-0.25)\n\
                    - ユーザーとの関連性 (0.0-0.25)\n\
                    - 新規性・独自性 (0.0-0.25)\n\
                    - 実用性 (0.0-0.25)\n\n\
                    スコアのみを小数で返してください。例: 0.75{}", context_info
                )),
                name: None,
                function_call: None,
            },
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                content: Some(format!("以下のテキストの重要度を評価してください:\n\n{}", content)),
                name: None,
                function_call: None,
            },
        ];

        let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
            .create()
            .await
            .context("Failed to create chat completion")?;

        let response = chat_completion
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_else(|| "0.5".to_string());

        // スコアを抽出（小数を含む数字）
        let score = response
            .trim()
            .parse::<f32>()
            .unwrap_or(0.5)
            .min(1.0)
            .max(0.0);

        Ok(score)
    }

    #[cfg(not(feature = "ai-analysis"))]
    pub async fn calculate_priority_score(&self, _content: &str, _user_context: Option<&str>) -> Result<f32> {
        Ok(0.5) // デフォルトスコア
    }

    /// AI解釈と心理判定を同時に実行
    pub async fn analyze(&self, content: &str, user_context: Option<&str>) -> Result<(String, f32)> {
        let interpreted = self.interpret_content(content).await?;
        let score = self.calculate_priority_score(content, user_context).await?;
        Ok((interpreted, score))
    }
}

impl Default for AIInterpreter {
    fn default() -> Self {
        Self::new()
    }
}
