use std::sync::Arc;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

#[derive(Serialize)]
struct RecipeView { id: Uuid, result_code: String, materials: serde_json::Value, gold_cost: i64 }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/crafting")
        .route("/recipes", web::get().to(list))
        .route("/fuse", web::post().to(fuse))
        .route("/craft/{recipe_id}", web::post().to(craft))
    );
}

async fn list(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let rows: Vec<(Uuid,String,serde_json::Value,i64)> = sqlx::query_as("SELECT id, result_code, materials, gold_cost FROM craft_recipes ORDER BY gold_cost").fetch_all(&state.db).await?;
    let out: Vec<RecipeView> = rows.into_iter().map(|(id,code,mats,cost)| RecipeView{ id, result_code:code, materials:mats, gold_cost:cost }).collect();
    Ok(HttpResponse::Ok().json(out))
}

// Fusão: 3× Comum→Incomum etc, 5× Mítico→Divino
#[derive(Deserialize)]
struct FuseRequest { template_code: String, quantity: i32 }

async fn fuse(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<FuseRequest>) -> AppResult<HttpResponse> {
    if body.quantity != 3 && body.quantity != 5 { return Err(AppError::Validation("fusão requer 3 ou 5 itens".into())); }
    let rarity: String = sqlx::query_scalar("SELECT rarity::text FROM item_templates WHERE code=$1").bind(&body.template_code).fetch_optional(&state.db).await?.ok_or(AppError::NotFound)?;
    let target_code = match (rarity.as_str(), body.quantity) {
        ("common",3) => "forest_bow_uncommon",
        ("uncommon",3) => "forest_staff_rare",
        ("rare",3) => "forest_armor_epic",
        ("epic",3) => "forest_relic_legendary",
        ("legendary",3) => "forest_crown_mythic",
        ("mythic",5) => "celestial_wings_divine",
        _ => return Err(AppError::Validation("combinação de fusão inválida".into())),
    };
    let mut tx = state.db.begin().await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM inventory_items WHERE user_id=$1 AND template_id=(SELECT id FROM item_templates WHERE code=$2)").bind(user.user_id).bind(&body.template_code).fetch_one(&mut *tx).await?;
    if count < body.quantity as i64 { return Err(AppError::Validation("itens insuficientes para fusão".into())); }
    // Remove 3 ou 5
    sqlx::query("DELETE FROM inventory_items WHERE id IN (SELECT id FROM inventory_items WHERE user_id=$1 AND template_id=(SELECT id FROM item_templates WHERE code=$2) LIMIT $3)")
        .bind(user.user_id).bind(&body.template_code).bind(body.quantity as i64).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO inventory_items (user_id, template_id) SELECT $1, id FROM item_templates WHERE code=$2").bind(user.user_id).bind(target_code).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'ITEM_FUSED',$2)").bind(user.user_id).bind(serde_json::json!({"from":body.template_code,"to":target_code,"qty":body.quantity})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"fused":target_code})))
}

async fn craft(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, recipe_id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let rid = recipe_id.into_inner();
    let mut tx = state.db.begin().await?;
    let row: Option<(String, serde_json::Value, i64)> = sqlx::query_as("SELECT result_code, materials, gold_cost FROM craft_recipes WHERE id=$1").bind(rid).fetch_optional(&mut *tx).await?;
    let (result_code, mats, cost) = row.ok_or(AppError::NotFound)?;
    let gold: i64 = sqlx::query_scalar("SELECT gold FROM users WHERE id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    if gold < cost { return Err(AppError::Validation(format!("gold insuficiente: precisa {cost}"))); }
    // Verifica materiais: mats é {"code": qty}
    if let Some(obj) = mats.as_object() {
        for (code, qty) in obj {
            let need = qty.as_i64().unwrap_or(0);
            let have: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM inventory_items WHERE user_id=$1 AND template_id=(SELECT id FROM item_templates WHERE code=$2)").bind(user.user_id).bind(code).fetch_one(&mut *tx).await?;
            if have < need { return Err(AppError::Validation(format!("material insuficiente: {code} precisa {need}"))); }
        }
        for (code, qty) in obj {
            let need = qty.as_i64().unwrap_or(0);
            sqlx::query("DELETE FROM inventory_items WHERE id IN (SELECT id FROM inventory_items WHERE user_id=$1 AND template_id=(SELECT id FROM item_templates WHERE code=$2) LIMIT $3)")
                .bind(user.user_id).bind(code).bind(need).execute(&mut *tx).await?;
        }
    }
    sqlx::query("UPDATE users SET gold=gold-$2 WHERE id=$1").bind(user.user_id).bind(cost).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO inventory_items (user_id, template_id) SELECT $1, id FROM item_templates WHERE code=$2").bind(user.user_id).bind(&result_code).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO craft_history (user_id, recipe_id) VALUES ($1,$2)").bind(user.user_id).bind(rid).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"crafted":result_code})))
}
