use std::future::Future;
use std::pin::Pin;
use crate::types::error::BirdeyeError;

#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl PaginationParams {
    pub fn new(offset: Option<u32>, limit: Option<u32>) -> Self {
        Self { offset, limit }
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            offset: None,
            limit: Some(10),
        }
    }
}

pub trait PaginatedRequest<T>: Send + Sync {
    type Future: Future<Output = Result<Vec<T>, BirdeyeError>> + Send;
    fn request(&self, offset: u32, limit: u32) -> Self::Future;
}

pub struct PaginatedIterator<T, R>
where
    R: PaginatedRequest<T>,
{
    request: R,
    current_offset: u32,
    page_size: u32,
    has_more: bool,
}

impl<T, R> PaginatedIterator<T, R>
where
    R: PaginatedRequest<T>,
{
    pub fn new(request: R, page_size: u32) -> Self {
        Self {
            request,
            current_offset: 0,
            page_size,
            has_more: true,
        }
    }

    pub async fn next_page(&mut self) -> Option<Result<Vec<T>, BirdeyeError>> {
        if !self.has_more {
            return None;
        }

        let result = self.request.request(self.current_offset, self.page_size).await;
        match result {
            Ok(items) => {
                self.has_more = items.len() as u32 == self.page_size;
                self.current_offset += items.len() as u32;
                Some(Ok(items))
            }
            Err(e) => {
                self.has_more = false;
                Some(Err(e))
            }
        }
    }

    pub async fn collect_all(mut self) -> Result<Vec<T>, BirdeyeError> {
        let mut all_items = Vec::new();

        while let Some(result) = self.next_page().await {
            all_items.extend(result?);
        }

        Ok(all_items)
    }
}
