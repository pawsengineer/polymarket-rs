/// Pagination cursors for API requests
pub const END_CURSOR: &str = "LTE=";
pub const INITIAL_CURSOR: &str = "MA==";

/// Pagination parameters for list endpoints
#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub next_cursor: Option<String>,
}

impl PaginationParams {
    pub fn new() -> Self {
        Self { next_cursor: None }
    }

    pub fn with_cursor(cursor: impl Into<String>) -> Self {
        Self {
            next_cursor: Some(cursor.into()),
        }
    }

    pub fn initial() -> Self {
        Self {
            next_cursor: Some(INITIAL_CURSOR.to_string()),
        }
    }

    pub fn to_query_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();
        if let Some(ref cursor) = self.next_cursor {
            params.push(("next_cursor", cursor.clone()));
        }
        params
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::initial();
        assert_eq!(params.next_cursor, Some(INITIAL_CURSOR.to_string()));

        let query = params.to_query_params();
        assert_eq!(query.len(), 1);
        assert_eq!(query[0].0, "next_cursor");
    }
}
