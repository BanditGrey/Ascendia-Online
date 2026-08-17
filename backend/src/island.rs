use std::sync::Arc;
use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/island")
        .route("/status", web::get().to(status))
        .route("/unlock", web::post().to(unlock))
        .route("/unlock/{island_code}", web::post().to(unlock_specific))
        .route("/enter/{stage}", web::post().to(enter))
    );
}

async fn status(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let max_stage: i16 = sqlx::query_scalar("SELECT COALESCE(max_stage,0)::smallint FROM stage_progress WHERE user_id=$1").bind(user.user_id).fetch_one(&state.db).await?;
    let gold: i64 = sqlx::query_scalar("SELECT gold FROM users WHERE id=$1").bind(user.user_id).fetch_one(&state.db).await?;
    let vip: i16 = sqlx::query_scalar("SELECT vip_level FROM users WHERE id=$1").bind(user.user_id).fetch_one(&state.db).await.unwrap_or(0);
    let awak: i16 = sqlx::query_scalar("SELECT COALESCE(MAX(awakening),0)::smallint FROM characters WHERE user_id=$1").bind(user.user_id).fetch_one(&state.db).await.unwrap_or(0);
    let prog_abyss: Option<(bool,i16)> = sqlx::query_as("SELECT unlocked, max_stage FROM island_progress WHERE user_id=$1 AND island_code='abyss_island'").bind(user.user_id).fetch_optional(&state.db).await?;
    let (unlocked_abyss, island_max_abyss) = prog_abyss.unwrap_or((false,500));
    let prog_gold: Option<(bool,i16)> = sqlx::query_as("SELECT unlocked, max_stage FROM island_progress WHERE user_id=$1 AND island_code='golden_kingdom'").bind(user.user_id).fetch_optional(&state.db).await?;
    let (unlocked_gold, island_max_gold) = prog_gold.unwrap_or((false,550));
    let prog_void: Option<(bool,i16)> = sqlx::query_as("SELECT unlocked, max_stage FROM island_progress WHERE user_id=$1 AND island_code='void_star'").bind(user.user_id).fetch_optional(&state.db).await?;
    let (unlocked_void, island_max_void) = prog_void.unwrap_or((false,600));
    let prog_eclipse: Option<(bool,i16)> = sqlx::query_as("SELECT unlocked, max_stage FROM island_progress WHERE user_id=$1 AND island_code='eclipse'").bind(user.user_id).fetch_optional(&state.db).await?;
    let (unlocked_eclipse, island_max_eclipse) = prog_eclipse.unwrap_or((false,650));
    let prog_storm: Option<(bool,i16)> = sqlx::query_as("SELECT unlocked, max_stage FROM island_progress WHERE user_id=$1 AND island_code='storm'").bind(user.user_id).fetch_optional(&state.db).await?;
    let (unlocked_storm, island_max_storm) = prog_storm.unwrap_or((false,700));
    let prog_time: Option<(bool,i16)> = sqlx::query_as("SELECT unlocked, max_stage FROM island_progress WHERE user_id=$1 AND island_code='time_labyrinth'").bind(user.user_id).fetch_optional(&state.db).await?;
    let (unlocked_time, island_max_time) = prog_time.unwrap_or((false,750));
    let prog_eternity: Option<(bool,i16)> = sqlx::query_as("SELECT unlocked, max_stage FROM island_progress WHERE user_id=$1 AND island_code='eternity'").bind(user.user_id).fetch_optional(&state.db).await?;
    let (unlocked_eternity, island_max_eternity) = prog_eternity.unwrap_or((false,800));
    let prog_origin: Option<(bool,i16)> = sqlx::query_as("SELECT unlocked, max_stage FROM island_progress WHERE user_id=$1 AND island_code='origin'").bind(user.user_id).fetch_optional(&state.db).await?;
    let (unlocked_origin, island_max_origin) = prog_origin.unwrap_or((false,850));
    Ok(HttpResponse::Ok().json(serde_json::json!([
        {"island":"abyss_island","name":"Ilha do Abismo Profundo","range":"501-550","theme":"Abismo aquático bioluminescente","requirement":"Fase 500 + 5000 Gold","max_stage":max_stage,"gold":gold,"can_unlock":max_stage>=500 && gold>=5000,"unlocked":unlocked_abyss,"island_max":island_max_abyss,"mobs":["abyssal_horror","deep_one","leviathan_spawn"],"boss":"Leviatã Ancião 550","loot":"Lâmina Abissal, Coração do Abismo, Coroa do Leviatã"},
        {"island":"golden_kingdom","name":"Reino Dourado","range":"551-600","theme":"Reino dourado flutuante","requirement":"Fase 550 + 8000 Gold + VIP 5","max_stage":max_stage,"vip":vip,"can_unlock":unlocked_abyss && max_stage>=550 && gold>=8000 && vip>=5,"unlocked":unlocked_gold,"island_max":island_max_gold,"mobs":["golden_golem","treasure_mimic","golden_phoenix"],"boss":"Rei Dourado 600","loot":"Lâmina Dourada, Armadura do Rei, Coroa Dourada Suprema"},
        {"island":"void_star","name":"Vazio Estelar","range":"601-650","theme":"Vazio estelar com cristais","requirement":"Fase 600 + 12000 Gold + VIP 8","max_stage":max_stage,"vip":vip,"can_unlock":unlocked_gold && max_stage>=600 && gold>=12000 && vip>=8,"unlocked":unlocked_void,"island_max":island_max_void,"mobs":["void_horror","star_wraith","void_spawn"],"boss":"Vazio Estelar 650","loot":"Lâmina do Vazio, Armadura Estelar, Coroa do Vazio"},
        {"island":"eclipse","name":"Eclipse Eterno","range":"651-700","theme":"Eclipse eterno com obeliscos","requirement":"Fase 650 + 15000 Gold + VIP 10 + Despertar 1","max_stage":max_stage,"vip":vip,"awak":awak,"can_unlock":unlocked_void && max_stage>=650 && gold>=15000 && vip>=10 && awak>=1,"unlocked":unlocked_eclipse,"island_max":island_max_eclipse,"mobs":["eclipse_horror","eclipse_wraith","eclipse_spawn"],"boss":"Eclipse Eterno 700","loot":"Lâmina do Eclipse, Armadura do Eclipse, Coroa do Eclipse Eterno"},
        {"island":"storm","name":"Tempestade Eterna","range":"701-750","theme":"Tempestade eterna com raios","requirement":"Fase 700 + 18000 Gold + VIP 12 + Despertar 2","max_stage":max_stage,"vip":vip,"awak":awak,"can_unlock":unlocked_eclipse && max_stage>=700 && gold>=18000 && vip>=12 && awak>=2,"unlocked":unlocked_storm,"island_max":island_max_storm,"mobs":["storm_horror","thunder_wraith","storm_spawn"],"boss":"Tempestade Eterna 750","loot":"Lâmina da Tempestade, Armadura da Tempestade, Coroa da Tempestade"},
        {"island":"time_labyrinth","name":"Labirinto do Tempo","range":"751-800","theme":"Labirinto temporal com relógios","requirement":"Fase 750 + 22000 Gold + VIP 13 + Despertar 3","max_stage":max_stage,"vip":vip,"awak":awak,"can_unlock":unlocked_storm && max_stage>=750 && gold>=22000 && vip>=13 && awak>=3,"unlocked":unlocked_time,"island_max":island_max_time,"mobs":["time_horror","chrono_wraith","time_spawn"],"boss":"Tempo Eterno 800","loot":"Lâmina do Tempo, Armadura Temporal, Coroa do Tempo Eterno"},
        {"island":"eternity","name":"Eternidade Dourada","range":"801-850","theme":"Eternidade dourada com trono","requirement":"Fase 800 + 26000 Gold + VIP 14 + Despertar 4","max_stage":max_stage,"vip":vip,"awak":awak,"can_unlock":unlocked_time && max_stage>=800 && gold>=26000 && vip>=14 && awak>=4,"unlocked":unlocked_eternity,"island_max":island_max_eternity,"mobs":["eternity_horror","eternity_wraith","eternity_spawn"],"boss":"Eternidade 850","loot":"Lâmina da Eternidade, Armadura da Eternidade, Coroa da Eternidade"},
        {"island":"origin","name":"Origem Primordial","range":"851-900","theme":"Origem primordial com o Criador","requirement":"Fase 850 + 30000 Gold + VIP 15 + Despertar 5 + Power 50000","max_stage":max_stage,"vip":vip,"awak":awak,"can_unlock":unlocked_eternity && max_stage>=850 && gold>=30000 && vip>=15 && awak>=5,"unlocked":unlocked_origin,"island_max":island_max_origin,"mobs":["origin_horror","origin_wraith","origin_spawn"],"boss":"O Criador 900","loot":"Lâmina da Origem, Armadura Primordial, Coroa do Criador"}
    ])))
}

async fn unlock(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    unlock_specific(state, user, web::Path::from("abyss_island".to_string())).await
}

async fn unlock_specific(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, island_code: web::Path<String>) -> AppResult<HttpResponse> {
    let code = island_code.into_inner();
    let allowed = ["abyss_island","golden_kingdom","void_star","eclipse","storm","time_labyrinth","eternity","origin"];
    if !allowed.contains(&code.as_str()) { return Err(AppError::Validation("ilha inválida".into())); }
    let mut tx = state.db.begin().await?;
    let max_stage: i16 = sqlx::query_scalar("SELECT COALESCE(max_stage,0)::smallint FROM stage_progress WHERE user_id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    let gold: i64 = sqlx::query_scalar("SELECT gold FROM users WHERE id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    let vip: i16 = sqlx::query_scalar("SELECT vip_level FROM users WHERE id=$1").bind(user.user_id).fetch_one(&mut *tx).await.unwrap_or(0);
    let awak: i16 = sqlx::query_scalar("SELECT COALESCE(MAX(awakening),0)::smallint FROM characters WHERE user_id=$1").bind(user.user_id).fetch_one(&mut *tx).await.unwrap_or(0);
    let power: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(power_rating),0)::bigint FROM character_stats cs JOIN characters c ON c.id=cs.character_id WHERE c.user_id=$1").bind(user.user_id).fetch_one(&mut *tx).await.unwrap_or(0);
    let (need_stage, need_gold, need_vip, need_awak, need_power, reward_skin) = match code.as_str() {
        "abyss_island" => (500,5000,0,0,0,"wings_t8_island_abyss"),
        "golden_kingdom" => (550,8000,5,0,0,"wings_t8_golden_kingdom"),
        "void_star" => (600,12000,8,0,0,"wings_t8_void_star"),
        "eclipse" => (650,15000,10,1,0,"wings_t8_eclipse"),
        "storm" => (700,18000,12,2,0,"wings_t8_storm"),
        "time_labyrinth" => (750,22000,13,3,0,"wings_t8_time"),
        "eternity" => (800,26000,14,4,0,"wings_t8_eternity"),
        "origin" => (850,30000,15,5,50000,"wings_t8_origin"),
        _ => (500,5000,0,0,0,"wings_t8_island_abyss"),
    };
    let prereq = match code.as_str() {
        "golden_kingdom" => "abyss_island",
        "void_star" => "golden_kingdom",
        "eclipse" => "void_star",
        "storm" => "eclipse",
        "time_labyrinth" => "storm",
        "eternity" => "time_labyrinth",
        "origin" => "eternity",
        _ => "",
    };
    if !prereq.is_empty() {
        let unlocked: bool = sqlx::query_scalar("SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=$1 AND island_code=$2), false)").bind(user.user_id).bind(prereq).fetch_one(&mut *tx).await?;
        if !unlocked { return Err(AppError::Validation(format!("complete {} primeiro", prereq))); }
    }
    if max_stage < need_stage { return Err(AppError::Validation(format!("requer Fase {need_stage}"))); }
    if gold < need_gold { return Err(AppError::Validation(format!("{need_gold} Gold necessários"))); }
    if vip < need_vip { return Err(AppError::Validation(format!("VIP {need_vip} necessário"))); }
    if awak < need_awak { return Err(AppError::Validation(format!("Despertar {need_awak} necessário"))); }
    if power < need_power { return Err(AppError::Validation(format!("Power {need_power} necessário"))); }
    let already: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM island_progress WHERE user_id=$1 AND island_code=$2 AND unlocked=true)").bind(user.user_id).bind(&code).fetch_one(&mut *tx).await?;
    if already { return Err(AppError::Validation("Ilha já desbloqueada".into())); }
    sqlx::query("UPDATE users SET gold=gold-$2 WHERE id=$1").bind(user.user_id).bind(need_gold as i64).execute(&mut *tx).await?;
    let start_stage = match code.as_str() {
        "abyss_island" => 501, "golden_kingdom" => 551, "void_star" => 601, "eclipse" => 651, "storm" => 701, "time_labyrinth" => 751, "eternity" => 801, "origin" => 851, _ => 501,
    };
    sqlx::query("INSERT INTO island_progress (user_id, island_code, unlocked, unlocked_at, max_stage) VALUES ($1,$2,true,now(),$3) ON CONFLICT (user_id,island_code) DO UPDATE SET unlocked=true, unlocked_at=now()")
        .bind(user.user_id).bind(&code).bind(start_stage).execute(&mut *tx).await?;
    let skin: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cosmetic_skins WHERE skin_code=$1").bind(reward_skin).fetch_optional(&mut *tx).await?;
    if let Some(sid)=skin { sqlx::query("INSERT INTO user_cosmetic_skins (user_id, skin_id) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(user.user_id).bind(sid).execute(&mut *tx).await?; }
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"unlocked":code,"stage":start_stage,"reward":reward_skin})))
}

async fn enter(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, stage: web::Path<u16>) -> AppResult<HttpResponse> {
    let s = stage.into_inner();
    if !(501..=900).contains(&s) { return Err(AppError::Validation("Ilhas 501-900 apenas".into())); }
    let code = if s<=550 {"abyss_island"} else if s<=600 {"golden_kingdom"} else if s<=650 {"void_star"} else if s<=700 {"eclipse"} else if s<=750 {"storm"} else if s<=800 {"time_labyrinth"} else if s<=850 {"eternity"} else {"origin"};
    let unlocked: bool = sqlx::query_scalar("SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=$1 AND island_code=$2), false)").bind(user.user_id).bind(code).fetch_one(&state.db).await?;
    if !unlocked { return Err(AppError::Validation(format!("desbloqueie {code} primeiro"))); }
    let (enemies, boss) = if s<=550 { (vec!["abyssal_horror","deep_one"], s==550) } else if s<=600 { (vec!["golden_golem","treasure_mimic"], s==600) } else if s<=650 { (vec!["void_horror","star_wraith"], s==650) } else if s<=700 { (vec!["eclipse_horror","eclipse_wraith"], s==700) } else if s<=750 { (vec!["storm_horror","thunder_wraith"], s==750) } else if s<=800 { (vec!["time_horror","chrono_wraith"], s==800) } else if s<=850 { (vec!["eternity_horror","eternity_wraith"], s==850) } else { (vec!["origin_horror","origin_wraith"], s==900) };
    let res: serde_json::Value = serde_json::json!({"stage":s,"island":code,"enemies":enemies,"boss":boss,"note":"Use POST /api/v1/combat/start com stage 501-900"});
    Ok(HttpResponse::Ok().json(res))
}
