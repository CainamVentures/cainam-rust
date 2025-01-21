use std::future::Future;
use std::pin::Pin;
use serde::{Deserialize, Serialize};
use crate::types::error::BirdeyeError;

/// Pagination parameters for API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub total: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            offset: Some(0),
            limit: Some(50), // Default page size
            total: None,
        }
    }
}

impl PaginationParams {
    pub fn new(offset: Option<u32>, limit: Option<u32>) -> Self {
        Self {
            offset,
            limit,
            total: None,
        }
    }

    pub fn with_total(mut self, total: u32) -> Self {
        self.total = Some(total);
        self
    }

    pub fn next_page(&self) -> Option<Self> {
        let current_offset = self.offset.unwrap_or(0);
        let limit = self.limit.unwrap_or(50);

        match self.total {
            Some(total) if current_offset + limit < total => Some(Self {
                offset: Some(current_offset + limit),
                limit: Some(limit),
                total: Some(total),
            }),
            None => Some(Self {
                offset: Some(current_offset + limit),
                limit: Some(limit),
                total: None,
            }),
            _ => None,
        }
    }

    pub fn has_more(&self) -> bool {
        match (self.offset, self.limit, self.total) {
            (Some(offset), Some(limit), Some(total)) => offset + limit < total,
            _ => true, // If we don't have total, assume there might be more
        }
    }
}

/// A paginated response from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationParams,
}

/// An async iterator over paginated results
pub struct PaginatedIterator<T, F, Fut> {
    fetch_page: F,
    current_pagination: Option<PaginationParams>,
    _phantom: std::marker::PhantomData<(T, Fut)>,
}

impl<T, F, Fut> PaginatedIterator<T, F, Fut>
where
    F: Fn(PaginationParams) -> Fut,
    Fut: Future<Output = Result<PaginatedResponse<T>, BirdeyeError>>,
{
    pub fn new(fetch_page: F) -> Self {
        Self {
            fetch_page,
            current_pagination: Some(PaginationParams::default()),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_params(fetch_page: F, params: PaginationParams) -> Self {
        Self {
            fetch_page,
            current_pagination: Some(params),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the next page of results
    pub async fn next_page(&mut self) -> Option<Result<Vec<T>, BirdeyeError>> {
        let pagination = self.current_pagination.take()?;
        
        match (self.fetch_page)(pagination.clone()).await {
            Ok(response) => {
                self.current_pagination = response.pagination.next_page();
                Some(Ok(response.data))
            }
            Err(e) => Some(Err(e)),
        }
    }

    /// Collect all pages into a single vector
    pub async fn collect_all(mut self) -> Result<Vec<T>, BirdeyeError> {
        let mut all_items = Vec::new();
        
        while let Some(result) = self.next_page().await {
            all_items.extend(result?);
        }
        
        Ok(all_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::default();
        assert_eq!(params.offset, Some(0));
        assert_eq!(params.limit, Some(50));
        assert_eq!(params.total, None);
    }

    #[test]
    fn test_pagination_next_page() {
        let params = PaginationParams::new(Some(0), Some(50))
            .with_total(150);
        
        let next = params.next_page().unwrap();
        assert_eq!(next.offset, Some(50));
        assert_eq!(next.limit, Some(50));
        assert_eq!(next.total, Some(150));
        
        let last = next.next_page().unwrap();
        assert_eq!(last.offset, Some(100));
        assert_eq!(last.limit, Some(50));
        assert_eq!(last.total, Some(150));
        
        assert!(last.next_page().is_none());
    }

    #[test]
    fn test_has_more() {
        let params = PaginationParams::new(Some(0), Some(50))
            .with_total(150);
        assert!(params.has_more());
        
        let params = PaginationParams::new(Some(100), Some(50))
            .with_total(150);
        assert!(!params.has_more());
        
        let params = PaginationParams::new(Some(0), Some(50));
        assert!(params.has_more());
    }

    #[tokio::test]
    async fn test_paginated_iterator() {
        let mut page_num = 0;
        let fetch_page = |params: PaginationParams| async move {
            page_num += 1;
            if page_num > 3 {
                return Ok(PaginatedResponse {
                    data: vec![],
                    pagination: params.with_total(150),
                });
            }
            
            Ok(PaginatedResponse {
                data: vec![1, 2, 3],
                pagination: params.with_total(150),
            })
        };

        let mut iterator = PaginatedIterator::new(fetch_page);
        
        let mut total_items = 0;
        while let Some(result) = iterator.next_page().await {
            let items = result.unwrap();
            total_items += items.len();
        }
        
        assert_eq!(total_items, 9); // 3 pages of 3 items each
    }
}
