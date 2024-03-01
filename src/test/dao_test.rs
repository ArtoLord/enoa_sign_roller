use anyhow::Result;
use testcontainers_modules::{postgres::Postgres, testcontainers::clients::Cli};
use crate::db::{psql, Dao, SignState, UserInfo};


// Global test scenario to reuse running psql container
// It is ugly, but rust not supports global state with cleanup
#[tokio::test]
async fn test_psql() -> Result<()> {
    // Setup

    let docker = Cli::default();

    let node = docker.run(Postgres::default());

    let dao = psql::init_with_config(deadpool_postgres::Config {
        user: Some("postgres".to_string()),
        password: Some("postgres".to_string()),
        dbname: Some("postgres".to_string()),
        host: Some("localhost".to_string()),
        port: Some(node.get_host_port_ipv4(5432)),
        ..Default::default()
    }).await?;

    // Running scenarios
    test_create_sign(&dao).await.unwrap();
    test_multi_create_sign(&dao).await.unwrap();
    test_get_guild(&dao).await.unwrap();
    test_change_sign_state(&dao).await.unwrap();
    test_user_info(&dao).await.unwrap();

    Ok(())
}


async fn test_create_sign(dao: &impl Dao) -> Result<()> {
    let guild = dao.create_sign(1, "sign".to_string(), 1).await?;

    assert!(guild.is_some());
    let g = guild.unwrap();
    
    assert_eq!(1, g.guild_id);
    assert_eq!("sign", g.current_sign.id);
    assert_eq!(1, g.current_sign.created_by_user_id);
    assert_eq!(SignState::Created, g.current_sign.state);
    Ok(())
}

async fn test_multi_create_sign(dao: &impl Dao) -> Result<()> {
    let guild = dao.create_sign(2, "sign".to_string(), 1).await?;

    assert!(guild.is_some());

    let guild2 = dao.create_sign(2, "sign".to_string(), 1).await?;
    assert!(guild2.is_none());

    Ok(())
}

async fn test_get_guild(dao: &impl Dao) -> Result<()> {
    let guild = dao.get_guild_info(3).await?;

    assert!(guild.is_none());

    let guild2 = dao.create_sign(3, "sign".to_string(), 1).await?;
    assert!(guild2.is_some());

    let guild3 = dao.get_guild_info(3).await?;
    assert!(guild3.is_some());

    let g = guild3.unwrap();
    
    assert_eq!(3, g.guild_id);
    assert_eq!("sign", g.current_sign.id);
    assert_eq!(1, g.current_sign.created_by_user_id);
    assert_eq!(SignState::Created, g.current_sign.state);

    Ok(())
}

async fn test_change_sign_state(dao: &impl Dao) -> Result<()> {
    let g = dao.change_sign_state(4, SignState::Success { by_user_id: 2 }).await?;
    assert!(g.is_err());
    assert!(g.err().unwrap().is_none());

    let g = dao.create_sign(4, "sign".to_string(), 1).await?;
    assert!(g.is_some());

    let g = dao.change_sign_state(4, SignState::Success { by_user_id: 1 }).await?;
    assert!(g.is_err());

    let g = dao.change_sign_state(4, SignState::Success { by_user_id: 2 }).await?;
    assert!(g.is_ok());

    let g = g.ok().unwrap();
    assert_eq!(4, g.guild_id);
    assert_eq!("sign", g.current_sign.id);
    assert_eq!(1, g.current_sign.created_by_user_id);
    assert_eq!(SignState::Success { by_user_id: 2 }, g.current_sign.state);

    let g = dao.change_sign_state(4, SignState::Success { by_user_id: 2 }).await?;
    assert!(g.is_err());

    Ok(())
}

async fn test_user_info(dao: &impl Dao) -> Result<()> {
    let u = dao.get_user_info(1, 1).await?;
    assert!(u.is_none());

    dao.save_user_info(UserInfo {id: 1, guild_id: 1, shaman_power: 10}).await?;

    let u = dao.get_user_info(1, 1).await?;
    assert!(u.is_some());

    let u = u.unwrap();
    assert_eq!(1, u.id);
    assert_eq!(1, u.guild_id);
    assert_eq!(10, u.shaman_power);
    Ok(())
}