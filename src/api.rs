use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct NewVideoResponse {
    vid: String,
    uid: String,
    timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct VideoDataResponse {
    vid: String,
    uid: String,
    preview: String,
    src: Option<String>,
    state: Option<String>,
}

pub async fn get_new_video_id(uid: &String) -> Result<String, reqwest::Error> {
    let new_video: NewVideoResponse = reqwest::Client::new()
        .get(format!(
            "http://{}/{}/{}",
            env!("API"),
            env!("API_NEW_VIDEO"),
            uid
        ))
        .send()
        .await?
        .json()
        .await?;

    return Ok(new_video.vid);
}

pub async fn get_preview_url(vid: &String) -> Result<String, reqwest::Error> {
    let video_data: VideoDataResponse = reqwest::Client::new()
        .get(format!(
            "http://{}/{}/{}",
            env!("API"),
            env!("API_VIDEO_DATA"),
            vid
        ))
        .send()
        .await?
        .json()
        .await?;

    return Ok(video_data.preview);
}

pub async fn cancel_recording(vid: &String) -> Result<(), reqwest::Error> {
    let _ = reqwest::Client::new()
        .delete(format!(
            "http://{}/{}/{}",
            env!("API"),
            env!("API_VIDEO_DATA"),
            vid
        ))
        .send()
        .await?
        .json()
        .await?;

    return Ok(());
}
