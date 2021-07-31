use futures::prelude::*;
use gotham::handler::HandlerFuture;
use gotham::helpers::http::response::create_empty_response;
use gotham::hyper::StatusCode;
use gotham::state::{FromState, State};
use gotham_middleware_jwt::AuthorizationToken;
use serde_derive::{Deserialize, Serialize};
use std::pin::Pin;
use validator::Validate;

use crate::auth::Claims;
use crate::db::Repo;

use crate::models::company::{delete_company, ChangeCompany, CompactCompany, NewCompany};
use crate::routes::paths::{PaginationExtractor, ResourceIDPath};
use crate::routes::utils::{
    extract_json, json_response_bad_message, json_response_created, json_response_not_found,
    json_response_ok,
};
use crate::sqlx::pagination::Paginate;

#[derive(Debug, Deserialize, Validate)]
pub struct NewCompanyRequest {
    pub name: String,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub country: Option<String>,
}

/// serve POST /api/v1/companies
/// this route create a company for logged in user
pub fn create_company_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let repo = Repo::borrow_from(&state).clone();

    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let current_user_id = token.0.claims.user_id();

    async move {
        let new_company = match extract_json::<NewCompanyRequest>(&mut state).await {
            Ok(company) => match company.validate() {
                Ok(_) => company,
                Err(e) => return Err((state, e.into())),
            },
            Err(e) => return Err((state, e)),
        };

        let result = repo
            .run(move |conn| {
                let new_company_db = &NewCompany {
                    user_id: current_user_id,
                    name: new_company.name,
                    address_1: new_company.address_1.unwrap_or(String::new()),
                    address_2: new_company.address_2.unwrap_or(String::new()),
                    city: new_company.city.unwrap_or(String::new()),
                    state: new_company.state.unwrap_or(String::new()),
                    zip_code: new_company.zip_code.unwrap_or(String::new()),
                    country: new_company.country.unwrap_or(String::new()),
                };
                new_company_db.insert_company(&conn)
            })
            .await;

        match result {
            Ok(company) => {
                let res = json_response_created(&state, &company);
                Ok((state, res))
            }
            Err(_) => {
                let res = json_response_bad_message(
                    &state,
                    "Unexpected error detected when trying to insert a company.".into(),
                );
                Ok((state, res))
            }
        }
    }
    .boxed()
}

/// serve PUT /api/v1/companies/:id
pub fn update_company_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let current_user_id = token.0.claims.user_id();

    let company_id = {
        let res = ResourceIDPath::borrow_from(&state);
        res.id
    };
    let repo = Repo::borrow_from(&state).clone();

    async move {
        let changes = match extract_json::<ChangeCompany>(&mut state).await {
            Ok(changes) => changes,
            Err(e) => return Err((state, e)),
        };

        let result = repo
            .run(move |conn| changes.update(current_user_id, company_id, &conn))
            .await;

        match result {
            Ok(client) => {
                let res = json_response_ok(&state, &client);
                Ok((state, res))
            }
            Err(_) => {
                let res = json_response_bad_message(
                    &state,
                    "Unexpected error detected when trying to update company.".into(),
                );
                Ok((state, res))
            }
        }
    }
    .boxed()
}

/// serve DELETE /api/v1/companies/:id
pub fn delete_company_handler(state: State) -> Pin<Box<HandlerFuture>> {
    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let current_user_id = token.0.claims.user_id();

    let company_id = {
        let res = ResourceIDPath::borrow_from(&state);
        res.id
    };
    let repo = Repo::borrow_from(&state).clone();

    async move {
        let result = repo
            .run(move |conn| delete_company(company_id, current_user_id, &conn))
            .await;

        match result {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    let res = create_empty_response(&state, StatusCode::NO_CONTENT);
                    Ok((state, res))
                } else {
                    let res = json_response_not_found(&state, "That resource is not found".into());
                    Ok((state, res))
                }
            }
            Err(_) => {
                let res = json_response_bad_message(
                    &state,
                    "Unexpected error detected when trying to update company.".into(),
                );
                Ok((state, res))
            }
        }
    }
    .boxed()
}

#[derive(Debug, Serialize, Deserialize)]
struct CompanyPagination {
    pub total_pages: i64,
    pub results: Vec<CompactCompany>,
}

/// serve GET /api/v1/companies
pub fn list_company_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let current_user_id = token.0.claims.user_id();
    let (per_page, page, search) = {
        let res = PaginationExtractor::take_from(&mut state);
        (res.per_page, res.page.unwrap_or(1), res.q)
    };
    let repo = Repo::borrow_from(&state).clone();

    async move {
        let result = repo
            .run(move |mut conn| {
                use crate::schema::companies;
                use crate::schema::companies::dsl::*;
                use diesel::prelude::*;

                let mut query = companies::table
                    .order(created_at.desc())
                    .filter(user_id.eq(current_user_id))
                    .select((id, name, created_at, updated_at))
                    .into_boxed();

                if let Some(search) = search {
                    query = query.filter(name.ilike(format!("{}%", search)));
                }

                let mut queryx = query.paginate(page);

                if let Some(per_page) = per_page {
                    use std::cmp::min;
                    queryx = queryx.per_page(min(per_page, 100));
                }

                queryx.load_and_count_pages::<CompactCompany>(&mut conn)
            })
            .await;

        match result {
            Ok((companies, total_pages)) => {
                let res = json_response_ok(
                    &state,
                    &CompanyPagination {
                        total_pages,
                        results: companies,
                    },
                );
                Ok((state, res))
            }
            Err(_) => {
                let res = json_response_bad_message(
                    &state,
                    "Unexpected error detected when trying to get companies".into(),
                );
                Ok((state, res))
            }
        }
    }
    .boxed()
}
