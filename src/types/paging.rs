use std::collections::HashMap;

use error_handler::AppError;

/// Paging struct used for pagination.
/// ## Example:
/// `?start=1&end=10`
#[derive(Debug, Default)]
pub struct Pagination {
    pub limit: Option<i32>,
    pub offset: i32,
}

pub fn extract_paging(params: HashMap<String, String>) -> Result<Pagination, AppError> {
    if params.contains_key("limit") && params.contains_key("offset") {
        let p = Pagination {
            limit: Some(
                params
                    .get("limit")
                    .unwrap()
                    .parse::<i32>()
                    .map_err(AppError::ParseError)?,
            ),
            offset: params
                .get("end")
                .unwrap()
                .parse::<i32>()
                .map_err(AppError::ParseError)?,
        };
        return Ok(p);
    }
    Err(AppError::MissingParams)
}
