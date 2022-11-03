use crate::{data::schedule::Page, SyncResult};
use super::error;

pub async fn merge(
    ft_daily: Page, 
    r_weekly: Page,
) -> SyncResult<Page> {

    let ft_date = ft_daily.date.start;

    if !r_weekly.date.contains(&ft_date) {
        return Err(error::FtDateIsNotInRWeeklyRange.into())
    }

    todo!()
}