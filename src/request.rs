use crate::error::{Error as WebError, Result};
// #[cfg(feature = "lua")]
// use crate::file_data::FileDataTrait;
#[cfg(feature = "lua")]
use crate::lua::file_data::FileData;
use http::{
    header::{self},
    HeaderMap, Method,
};
use hyper::{Body, Request as HyperRequest};
#[cfg(feature = "lua")]
use multer::Multipart;
use serde_json::Value as JsonValue;
use std::{collections::HashMap, net::SocketAddr};

pub struct Request {
    pub req: HyperRequest<Body>,
    pub remote_addr: SocketAddr,
}

pub type HttpData<T> = HashMap<String, T>;

fn has_content_type(headers: &HeaderMap, expected_content_type: &mime::Mime) -> bool {
    let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    content_type.starts_with(expected_content_type.as_ref())
}

impl Request {
    pub async fn params<T, F1, F2>(self, mut f1: F1, mut f2: F2) -> Result<HttpData<T>>
    where
        T: Clone,
        F1: FnMut(HttpData<T>, String, Vec<String>, JsonValue) -> Result<HttpData<T>>, // 用于处理多维数组参数
        F2: FnMut(HttpData<T>, String, JsonValue) -> Result<HttpData<T>>, // 用于处理正常参数
    {
        let mut param: HttpData<T> = HashMap::new();
        if self.req.method() == Method::GET {
            let query = self.req.uri().query().unwrap_or_default();
            let value = serde_urlencoded::from_str::<Vec<(String, JsonValue)>>(query)
                .map_err(WebError::parse_params)?;

            for (key, val) in value {
                let left_square_bracket = key.find('[');
                if let Some(l) = left_square_bracket {
                    let param_name = key.get(0..l);
                    if let Some(param_key) = param_name {
                        let right_square_bracket = key.rfind(']');
                        if let Some(r) = right_square_bracket {
                            let field_str = key.get((l + 1)..r);
                            if let Some(field_str) = field_str {
                                let fields: Vec<&str> = field_str.split("][").collect();
                                let fields = fields.iter().map(|v| v.to_string()).collect();
                                param = f1(param, param_key.to_string(), fields, val)?;
                            } else {
                                return Err(WebError::new(
                                    5031,
                                    "The transmitted parameters are incorrect",
                                ));
                            }
                        } else {
                            return Err(WebError::new(
                                5031,
                                "The transmitted parameters are incorrect",
                            ));
                        }
                    }
                } else {
                    param = f2(param, key, val)?;
                }
            }
        } else {
            if !has_content_type(self.req.headers(), &mime::APPLICATION_WWW_FORM_URLENCODED) {
                return Ok(param);
            }
            let bytes = hyper::body::to_bytes(self.req).await?;
            let value = serde_urlencoded::from_bytes::<Vec<(String, JsonValue)>>(&bytes)
                .map_err(WebError::parse_params)?;

            for (key, val) in value {
                let left_square_bracket = key.find('[');
                if let Some(l) = left_square_bracket {
                    let param_name = key.get(0..l);
                    if let Some(param_key) = param_name {
                        let right_square_bracket = key.rfind(']');
                        if let Some(r) = right_square_bracket {
                            let field_str = key.get((l + 1)..r);
                            if let Some(field_str) = field_str {
                                let fields: Vec<&str> = field_str.split("][").collect();
                                let fields = fields.iter().map(|v| v.to_string()).collect();
                                param = f1(param, param_key.to_string(), fields, val)?;
                            } else {
                                return Err(WebError::new(
                                    5031,
                                    "The transmitted parameters are incorrect",
                                ));
                            }
                        } else {
                            return Err(WebError::new(
                                5031,
                                "The transmitted parameters are incorrect",
                            ));
                        }
                    }
                } else {
                    param = f2(param, key, val)?;
                }
            }
        }
        Ok(param)
    }

    pub async fn form<T, F1, F2, F3>(self, file_func: F1, f1: F2, f2: F3) -> Result<HttpData<T>>
    where
        T: Clone,
        F1: Fn(HttpData<T>, String, FileData) -> Result<HttpData<T>>,
        F2: Fn(HttpData<T>, String, Vec<String>, JsonValue) -> Result<HttpData<T>>,
        F3: Fn(HttpData<T>, String, JsonValue) -> Result<HttpData<T>>,
    {
        let mut param: HttpData<T> = HttpData::new();
        if !has_content_type(self.req.headers(), &mime::MULTIPART_FORM_DATA) {
            return Ok(param);
        }
        let boundary = self
            .req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .and_then(|ct| multer::parse_boundary(ct).ok());

        if boundary.is_none() {
            return Err(WebError::new(5041, "no multipart boundary was found"));
        }

        let mut multipart = Multipart::new(self.req.into_body(), boundary.unwrap());

        while let Some(mut field) = multipart.next_field().await? {
            let name = field.name().map(|v| v.to_string());

            let file_name = field.file_name().map(|v| v.to_string());

            let content_type = field.content_type().map(|v| v.to_string());

            let mut field_data: Vec<u8> = Vec::new();
            while let Some(field_chunk) = field.chunk().await? {
                field_data.append(&mut field_chunk.to_vec());
            }

            if let Some(file_name) = file_name.clone() {
                let field_name = name.clone().unwrap_or_else(|| "default".to_string());
                let content_type = content_type
                    .clone()
                    .unwrap_or_else(|| "image/jpeg".to_string());
                let file = FileData::new(field_name.clone(), file_name, content_type, field_data);
                param = file_func(param, field_name, file)?;
            } else if let Some(field_name) = name.clone() {
                // let data = String::from_utf8(field_data)?;
                let data = JsonValue::from(field_data);
                let left_square_bracket = field_name.find('[');
                if let Some(l) = left_square_bracket {
                    let param_name = field_name.get(0..l);
                    if let Some(param_key) = param_name {
                        let right_square_bracket = field_name.rfind(']');
                        if let Some(r) = right_square_bracket {
                            let field_str = field_name.get((l + 1)..r);
                            if let Some(field_str) = field_str {
                                let fields: Vec<&str> = field_str.split("][").collect();
                                let fields = fields.iter().map(|v| v.to_string()).collect();
                                param = f1(param, param_key.to_string(), fields, data)?;
                            } else {
                                return Err(WebError::new(
                                    5031,
                                    "The transmitted parameters are incorrect",
                                ));
                            }
                        } else {
                            return Err(WebError::new(
                                5031,
                                "The transmitted parameters are incorrect",
                            ));
                        }
                    }
                } else {
                    param = f2(param, field_name, data)?;
                }
            }
        }
        Ok(param)
    }
}
