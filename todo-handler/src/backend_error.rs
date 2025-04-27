use salvo::prelude::*;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum BackendError {
    #[error("Environment variable error: {0}")]
    EnvError(String),

    #[error("Request error: {0:?}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Sqlx error: {0:?}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Salvo parse error: {0:?}")]
    SalvoParseError (#[from] salvo::http::ParseError),
}

#[async_trait]
impl Writer for BackendError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(format!("{:?}", self));
    }
}