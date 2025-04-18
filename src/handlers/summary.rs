use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use axum_extra::extract::WithRejection;
use std::sync::Arc;

use crate::{
    common::errors::AppError, dtos::summary::GenerateSummaryRequest, repositories::summary,
};

/// Handles the routes related to summary operations.
pub fn summary_routes() -> Router<Arc<dyn summary::RepositoryOperation>> {
    Router::new().nest(
        "/summaries/generate",
        Router::new().route("/raw", post(generate)),
    )
}

/// Handles the generation of a summary based on request parameters.
async fn generate(
    State(summary_repository): State<Arc<dyn summary::RepositoryOperation>>,
    WithRejection(Json(body), _): WithRejection<Json<GenerateSummaryRequest>, AppError>,
) -> Result<impl IntoResponse, AppError> {
    let summary = summary_repository.generate_raw(&body).await?;

    Ok((StatusCode::OK, Json(summary)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dtos::{
        query_result::{
            ExpenseGroupedSummary, ExpenseParentCategory, ExpensePriority, ExpenseSummary,
            IncomeGroupedSummary, IncomeSummary, ShowSummary, SimpleAmountEntity,
        },
        summary::GenerateSummaryRequest,
    };

    use async_trait::async_trait;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json;
    use sqlx::Error as SqlxError;
    use std::sync::Arc;
    use tower::ServiceExt;

    pub struct MockSummaryRepository;

    impl MockSummaryRepository {
        pub fn new() -> Arc<Self> {
            Arc::new(Self)
        }
    }

    fn show_summary_response() -> ShowSummary {
        ShowSummary {
            expense: sqlx::types::Json(ExpenseSummary {
                amount: 5000,
                group_summary: ExpenseGroupedSummary {
                    parent_categories: vec![
                        ExpenseParentCategory {
                            name: "Daily Expenses".to_string(),
                            amount: 3000,
                            categories: vec![
                                SimpleAmountEntity {
                                    name: "Food".to_string(),
                                    amount: 2000,
                                },
                                SimpleAmountEntity {
                                    name: "Transportation".to_string(),
                                    amount: 1000,
                                },
                            ],
                        },
                        ExpenseParentCategory {
                            name: "Monthly Bills".to_string(),
                            amount: 2000,
                            categories: vec![
                                SimpleAmountEntity {
                                    name: "Rent".to_string(),
                                    amount: 1500,
                                },
                                SimpleAmountEntity {
                                    name: "Utilities".to_string(),
                                    amount: 500,
                                },
                            ],
                        },
                    ],
                    priorities: vec![
                        ExpensePriority {
                            level: 0,
                            amount: 2500,
                        },
                        ExpensePriority {
                            level: 1,
                            amount: 1500,
                        },
                        ExpensePriority {
                            level: 2,
                            amount: 1000,
                        },
                    ],
                },
            }),
            income: sqlx::types::Json(IncomeSummary {
                amount: 8000,
                group_summary: IncomeGroupedSummary {
                    wallets: vec![
                        SimpleAmountEntity {
                            name: "Salary".to_string(),
                            amount: 6000,
                        },
                        SimpleAmountEntity {
                            name: "Freelance".to_string(),
                            amount: 2000,
                        },
                    ],
                },
            }),
        }
    }

    #[async_trait]
    impl summary::RepositoryOperation for MockSummaryRepository {
        async fn generate_raw(
            &self,
            _request: &GenerateSummaryRequest,
        ) -> Result<ShowSummary, SqlxError> {
            Ok(show_summary_response())
        }
    }

    #[tokio::test]
    async fn test_generate_handler() {
        // Prepare
        let repo = MockSummaryRepository::new();
        let app = summary_routes().with_state(repo);

        let request = Request::builder()
            .method("POST")
            .uri("/summaries/generate/raw")
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "startDate": "2025-03-01",
                    "endDate": "2025-04-01",
                    "excludeCategoryIds": [5, 10]
                })
                .to_string(),
            ))
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<ShowSummary>(&body_bytes).unwrap();

        assert_eq!(body.expense.amount, show_summary_response().expense.amount);
        assert_eq!(body.income.amount, show_summary_response().income.amount);
    }
}
