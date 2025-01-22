use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            offset: None,
            limit: Some(10),
        }
    }
}

impl PaginationParams {
    pub fn new(offset: Option<u32>, limit: Option<u32>) -> Self {
        Self { offset, limit }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationParams,
}

pub struct PaginatedIterator<T, F, Fut>
where
    F: Fn(PaginationParams) -> Fut,
    Fut: std::future::Future<Output = Result<PaginatedResponse<T>, crate::types::error::BirdeyeError>>,
{
    fetch: F,
    current_page: Option<PaginationParams>,
    page_size: u32,
}

impl<T, F, Fut> PaginatedIterator<T, F, Fut>
where
    F: Fn(PaginationParams) -> Fut,
    Fut: std::future::Future<Output = Result<PaginatedResponse<T>, crate::types::error::BirdeyeError>>,
{
    pub fn new(fetch: F) -> Self {
        Self {
            fetch,
            current_page: Some(PaginationParams::default()),
            page_size: 10,
        }
    }

    pub async fn next_page(&mut self) -> Option<Result<Vec<T>, crate::types::error::BirdeyeError>> {
        if let Some(params) = self.current_page.take() {
            match (self.fetch)(params.clone()).await {
                Ok(response) => {
                    if response.data.is_empty() {
                        return None;
                    }

                    self.current_page = Some(PaginationParams {
                        offset: Some(params.offset.unwrap_or(0) + self.page_size),
                        limit: Some(self.page_size),
                    });

                    Some(Ok(response.data))
                }
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }

    pub async fn collect_all(mut self) -> Result<Vec<T>, crate::types::error::BirdeyeError>
    where
        T: Clone,
    {
        let mut all_items = Vec::new();
        while let Some(result) = self.next_page().await {
            match result {
                Ok(items) => all_items.extend(items),
                Err(e) => return Err(e),
            }
        }
        Ok(all_items)
    }
}
