#![allow(unused)]

use crate::db::Database;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, sqlx::Type, PartialEq, Clone)]
#[sqlx(type_name = "request_status", rename_all = "lowercase")]
pub enum RequestStatus {
    Success,
    Redirect,
    ClientError,
    ServerError,
    Unauthorized,
}

#[derive(Debug, sqlx::FromRow)]
pub struct WebsiteTrace {
    pub trace_id: Uuid,
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub user_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub status_code: u16,
    pub request_status: RequestStatus,
    pub data: serde_json::Value,
    pub request_headers: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub response_body: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl WebsiteTrace {
    pub fn start(
        method: impl AsRef<str>,
        path: impl Into<String>,
        query: Option<impl Into<String>>,
        user_agent: Option<impl Into<String>>,
        ip: Option<impl Into<String>>,
    ) -> Self {
        Self {
            trace_id: Uuid::now_v7(),
            method: method.as_ref().to_uppercase(),
            path: path.into(),
            query: query.map(Into::into),
            ip_address: ip.map(Into::into),
            user_agent: user_agent.map(Into::into),
            user_id: None,
            started_at: Utc::now(),
            duration_ms: 0,
            status_code: 0,
            data: serde_json::json!({}),
            request_headers: serde_json::json!({}),
            request_status: RequestStatus::Success,
            request_body: None,
            response_body: None,
            error: None,
        }
    }

    pub fn complete(&mut self, duration_ms: i64, status_code: u16, user_id: Option<Uuid>) {
        self.duration_ms = duration_ms;
        self.status_code = status_code;
        self.user_id = user_id;
    }
}

impl Database {
    pub async fn save_website_trace(&self, trace: &WebsiteTrace) -> Result<Uuid, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            INSERT INTO website_traces (
                trace_id, method, path, query, ip_address, user_agent, user_id, started_at,
                duration_ms, status_code, data, request_headers, request_status, request_body,
                response_body, error
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8,
                $9, $10, $11, $12, $13, $14,
                $15, $16
            )
            "#,
            trace.trace_id,
            trace.method,
            trace.path,
            trace.query,
            trace.ip_address,
            trace.user_agent,
            trace.user_id,
            trace.started_at,
            trace.duration_ms,
            trace.status_code as i32,
            trace.data,
            trace.request_headers,
            trace.request_status.clone() as RequestStatus,
            trace.request_body,
            trace.response_body,
            trace.error
        )
        .execute(self.pool())
        .await?;

        Ok(trace.trace_id)
    }
}
