mod error;

pub mod api;
pub mod dto;


#[cfg(test)]
mod tests {
    use super::{error::{VkResult, VkError}, dto::VkProfileData};

    #[test]
    fn vk_response_ok_parse() {
        let json = serde_json::json!({
            "status": "ok"
        })
        .to_string();

        let data = serde_json::from_str::<VkResult<()>>(&json);
        assert!(data.is_ok());

        let data = data.unwrap();

        assert_eq!(data, VkResult::Ok(()));
    }

    #[test]
    fn vk_response_structure_parse() {
        let json = serde_json::json!({
            "status": "ok",
            "uid": 0,
            "nick": "test",
            "avatar": "http://test.com/test",
            "birthyear": "01.01.2001",
            "sex": "male",
            "slug": "test"
        })
        .to_string();

        let data = serde_json::from_str::<VkResult<VkProfileData>>(&json);
        assert!(data.is_ok());

        let data = data.unwrap();

        assert_eq!(
            data,
            VkResult::Ok(VkProfileData {
                uid: 0,
                nick: "test".to_string(),
                avatar: "http://test.com/test".to_string(),
                birthyear: "01.01.2001".to_string(),
                sex: "male".to_string(),
                slug: "test".to_string()
            })
        );
    }

    #[test]
    fn vk_response_error_parse() {
        let json = serde_json::json!({
            "status": "error",
            "errcode": 0,
            "errmsg": "gas_invalid_sign"
        })
        .to_string();

        let data = serde_json::from_str::<VkResult<()>>(&json);
        assert!(data.is_ok());

        let data = data.unwrap();

        assert_eq!(
            data,
            VkResult::Err(VkError {
                errcode: 0,
                errmsg: "gas_invalid_sign".to_string()
            })
        );
    }
}
