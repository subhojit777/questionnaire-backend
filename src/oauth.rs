use crate::*;
use actix_web::{HttpRequest, HttpResponse, Error};
use futures::Future;
use oxide_auth::code_grant::frontend::OAuthError;
use oxide_auth::frontends::actix::OAuth;
use oxide_auth::frontends::actix::ResolvedResponse;
use oxide_auth::primitives::prelude::PreGrant;
use oxide_auth::code_grant::frontend::OwnerAuthorization;

pub fn authorize_get(req: &HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let state: AppState = req.state().clone();

    Box::new(req.oauth2()
        .authorization_code(handle_get)
        .and_then(move |request| state
            .endpoint
            .send(request)
            .map_err(|_| OAuthError::InvalidRequest)
            .and_then(|result| result.map(Into::into))
        )
        .or_else(|err| Ok(ResolvedResponse::response_or_error(err).actix_response()))
    )
}

pub fn authorize_post(req: &HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let state: AppState = req.state().clone();
    let denied = req.query_string().contains("deny");

    Box::new(req.oauth2()
        .authorization_code(move |grant| handle_post(denied, grant))
        .and_then(move |request| state
            .endpoint
            .send(request)
            .map_err(|_| OAuthError::InvalidRequest)
            .and_then(|result| result.map(Into::into))
        )
        .or_else(|err| Ok(ResolvedResponse::response_or_error(err).actix_response()))
    )
}

fn handle_get(grant: &PreGrant) -> OwnerAuthorization<ResolvedResponse> {
    let text = format!(
        "<html>'{}' (at {}) is requesting permission for '{}'\
        <form method=\"post\">\
            <input type=\"submit\" value=\"Accept\" formaction=\"authorize?response_type=code&client_id={}\">\
            <input type=\"submit\" value=\"Deny\" formaction=\"authorize?response_type=code&client_id={}&deny=1\">\
        </form>\
        </html>", grant.client_id, grant.redirect_uri, grant.scope, grant.client_id, grant.client_id);
    let response = ResolvedResponse::html(&text);
    OwnerAuthorization::InProgress(response)
}

fn handle_post(denied: bool, _: &PreGrant) -> OwnerAuthorization<ResolvedResponse> {
    if denied {
        OwnerAuthorization::Denied
    } else {
        OwnerAuthorization::Authorized("dummy user".to_string())
    }
}
