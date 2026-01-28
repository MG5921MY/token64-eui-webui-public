/*!
 * 认证中间件
 * 
 * 保护 API 端点，要求有效的会话
 */

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::sync::Arc;
use std::rc::Rc;
use log::warn;

use crate::auth::AuthManager;
use crate::models::ApiResponse;

/// 认证中间件
pub struct AuthMiddleware {
    auth_manager: Arc<AuthManager>,
}

impl AuthMiddleware {
    pub fn new(auth_manager: Arc<AuthManager>) -> Self {
        Self { auth_manager }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            auth_manager: self.auth_manager.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    auth_manager: Arc<AuthManager>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        
        // 排除不需要认证的路径
        if path.starts_with("/api/auth/") || path == "/" || !path.starts_with("/api/") {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }
        
        // 检查是否需要初始化密码
        if self.auth_manager.needs_password_setup() {
            warn!("尝试访问受保护的 API，但密码未设置: {}", path);
            let response = HttpResponse::Unauthorized().json(
                ApiResponse::<()>::error("请先设置初始密码".to_string())
            );
            return Box::pin(async move {
                Ok(req.into_response(response).map_into_right_body())
            });
        }
        
        // 提取会话 ID
        let session_id = req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string());
        
        let auth_manager = self.auth_manager.clone();
        let service = self.service.clone();
        
        // 验证会话
        Box::pin(async move {
            if let Some(sid) = session_id {
                match auth_manager.validate_session(&sid) {
                    Ok(true) => {
                        // 会话有效，继续处理请求
                        let res = service.call(req).await?;
                        Ok(res.map_into_left_body())
                    }
                    Ok(false) => {
                        // 会话无效或过期
                        warn!("会话无效或已过期: {}", sid);
                        let response = HttpResponse::Unauthorized().json(
                            ApiResponse::<()>::error("会话已过期，请重新登录".to_string())
                        );
                        Ok(req.into_response(response).map_into_right_body())
                    }
                    Err(e) => {
                        // 验证出错
                        warn!("会话验证失败: {}", e);
                        let response = HttpResponse::InternalServerError().json(
                            ApiResponse::<()>::error("认证失败，请重试".to_string())
                        );
                        Ok(req.into_response(response).map_into_right_body())
                    }
                }
            } else {
                // 没有提供会话 ID
                warn!("未授权访问: {} (缺少 Authorization header)", path);
                let response = HttpResponse::Unauthorized().json(
                    ApiResponse::<()>::error("未授权访问，请先登录".to_string())
                );
                Ok(req.into_response(response).map_into_right_body())
            }
        })
    }
}
