use archivum_core::blob::BlobStore;
use reqwest::Client;

pub struct NetworkBlobStore {
    remote_url: reqwest::Url,
}

impl NetworkBlobStore {
    pub fn new(remote_url: String) -> Self {
        let normalized_url = if remote_url.ends_with('/') {
            remote_url
        } else {
            format!("{}/", remote_url)
        };

        NetworkBlobStore { remote_url: reqwest::Url::parse(&normalized_url).unwrap() }
    }
}

impl BlobStore for NetworkBlobStore {
    type Error = NetworkBlobStoreError;

    async fn upload(
        &mut self,
        blob_id: &archivum_core::blob::BlobId,
        data: &[u8]
    ) -> Result<(), Self::Error> {
        let client = Client::new();

        let res = client
            .post(self.remote_url.join(&blob_id.to_string()).unwrap())
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec())
            .send().await;

        match res {
            Ok(response) => {
                let status = response.status();

                if status.is_success() {
                    Ok(())
                } else {
                    Err(
                        NetworkBlobStoreError::NetworkError(
                            format!("upload failed with status code: {}", status)
                        )
                    )
                }
            }
            Err(err) =>
                Err(NetworkBlobStoreError::NetworkError(format!("upload failed: {:?}", err))),
        }
    }

    async fn download(
        &self,
        blob_id: &archivum_core::blob::BlobId
    ) -> Result<Vec<u8>, Self::Error> {
        let client = Client::new();

        let res = client.get(self.remote_url.join(&blob_id.to_string()).unwrap()).send().await;

        match res {
            Ok(response) if response.status().is_success() => {
                let bytes = response
                    .bytes().await
                    .map_err(|_|
                        NetworkBlobStoreError::NetworkError(
                            String::from("Failed to read response bytes")
                        )
                    )?;
                Ok(bytes.to_vec())
            }
            _ => Err(NetworkBlobStoreError::NetworkError(String::from("Failed to download blob"))),
        }
    }

    async fn check_exists(
        &self,
        blob_id: &archivum_core::blob::BlobId
    ) -> Result<bool, Self::Error> {
        let client = Client::new();
        let res = client.head(self.remote_url.join(&blob_id.to_string()).unwrap()).send().await;
        match res {
            Ok(response) => Ok(response.status().is_success()),
            Err(err) =>
                Err(NetworkBlobStoreError::NetworkError(format!("check_exists failed: {:?}", err))),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum NetworkBlobStoreError {
    #[error("Network error: {0}")] NetworkError(String),
}
