use serde_json::json;

use crate::security::hash_password;
use crate::state::AppState;

struct SeedUser {
    username: &'static str,
    password: &'static str,
    email: &'static str,
    role: &'static str,
    bio: &'static str,
}

const USERS: [SeedUser; 3] = [
    SeedUser {
        username: "admin",
        password: "admin123",
        email: "admin@openforum.local",
        role: "ADMIN",
        bio: "OpenForum 站点管理员",
    },
    SeedUser {
        username: "alice",
        password: "alice123",
        email: "alice@openforum.local",
        role: "USER",
        bio: "热爱技术与生活的普通用户",
    },
    SeedUser {
        username: "bob",
        password: "bob123",
        email: "bob@openforum.local",
        role: "USER",
        bio: "潜水多年的老用户",
    },
];

/// 对应 Java `DataSeeder.run`：仅在 `sub_forums` 为空时执行。
pub async fn seed(state: &AppState) -> anyhow::Result<()> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sub_forums")
        .fetch_one(&state.pool)
        .await?;
    if count > 0 {
        tracing::info!("种子数据已存在，跳过 DataSeeder");
        return Ok(());
    }

    // 对应 Java `DataSeeder.run` 上的 `@Transactional`：种子数据整体原子落库，
    // 中途失败则全部回滚，避免留下半成品（例如有论坛无成员/计数不一致）。
    let mut tx = state.pool.begin().await?;

    for u in USERS.iter() {
        ensure_user(&mut tx, u).await?;
    }

    seed_authentication(&mut tx, "alice", "REAL_NAME", true, json!({"fullName": "爱丽丝"}))
        .await?;
    seed_authentication(&mut tx, "alice", "AGE", true, json!({"age": 25})).await?;
    seed_authentication(&mut tx, "bob", "REAL_NAME", true, json!({"fullName": "鲍勃"}))
        .await?;
    seed_authentication(&mut tx, "bob", "NATIONALITY", false, json!({"nationality": "中国"}))
        .await?;

    let admin_id = user_id(&mut tx, "admin").await?;
    let alice_id = user_id(&mut tx, "alice").await?;
    let bob_id = user_id(&mut tx, "bob").await?;

    let general = create_forum(
        &mut tx,
        "general",
        "综合讨论区",
        "任何话题都可以在这里畅聊",
        "#FF4500",
        None,
        admin_id,
        &[(alice_id, "MEMBER"), (bob_id, "MEMBER")],
    )
    .await?;
    let tech = create_forum(
        &mut tx,
        "tech",
        "技术交流",
        "编程、架构与开源技术分享",
        "#3498DB",
        None,
        admin_id,
        &[(alice_id, "MEMBER")],
    )
    .await?;
    let life = create_forum(
        &mut tx,
        "life",
        "生活百态",
        "记录生活的点点滴滴",
        "#2ECC71",
        None,
        admin_id,
        &[(bob_id, "MEMBER")],
    )
    .await?;
    let adult = create_forum(
        &mut tx,
        "adult",
        "成人话题",
        "仅限实名且年满 18 周岁的用户参与",
        "#9B59B6",
        Some("REAL_NAME,AGE_18"),
        admin_id,
        &[(alice_id, "MEMBER")],
    )
    .await?;
    let creative = create_forum(
        &mut tx,
        "creative",
        "创意工坊",
        "展示你的创作与灵感",
        "#E67E22",
        None,
        admin_id,
        &[(alice_id, "MEMBER")],
    )
    .await?;

    // (forum, author, title, content, pinned)
    let posts: [(i64, i64, &str, &str, bool); 10] = [
        (general, admin_id, "欢迎来到 OpenForum！", "这是一个开放式的社区原型，欢迎自由发帖、评论与投票。", true),
        (general, alice_id, "新人报到，请多关照", "大家好，我是 alice，喜欢折腾各种新技术。", false),
        (tech, alice_id, "Spring Boot 4 有哪些值得关注的变化？", "想和大家讨论一下 Spring Boot 4 与 Spring Framework 7 的新特性。", true),
        (tech, admin_id, "MySQL 索引优化的一些实践经验", "记录几个常见的索引失效场景与优化思路。", false),
        (life, bob_id, "周末去了趟郊外，风景很好", "分享几张周末出游拍的照片，空气真的很清新。", false),
        (life, bob_id, "分享一家宝藏咖啡馆", "藏在小巷子里，手冲非常不错，环境也很安静。", false),
        (adult, admin_id, "成人话题区使用须知", "本区仅限完成实名认证且年满 18 周岁的用户参与，请勿发布违规内容。", true),
        (adult, alice_id, "成年人的理财心得", "聊聊定投、指数基金和风险控制的一些个人看法。", false),
        (creative, alice_id, "分享我的插画练习作品", "最近在练习厚涂，放几张过程图，欢迎交流。", true),
        (creative, alice_id, "写了一篇短篇科幻小说", "关于时间循环的一个小故事，附上全文，欢迎点评。", false),
    ];

    let mut created_post_ids: Vec<(usize, i64)> = Vec::new();
    for (idx, (forum_id, author_id, title, content, pinned)) in posts.iter().enumerate() {
        let result = sqlx::query(
            "INSERT INTO posts (forum_id, author_id, title, content, upvotes, downvotes, score, comment_count, \
             is_pinned, is_removed, created_at, updated_at) VALUES (?, ?, ?, ?, 0, 0, 0, 0, ?, 0, NOW(), NOW())",
        )
        .bind(forum_id)
        .bind(author_id)
        .bind(title)
        .bind(content)
        .bind(if *pinned { 1 } else { 0 })
        .execute(&mut *tx)
        .await?;
        created_post_ids.push((idx, result.last_insert_id() as i64));
    }

    let pid = |idx: usize| -> i64 {
        created_post_ids
            .iter()
            .find(|(i, _)| *i == idx)
            .map(|(_, id)| *id)
            .unwrap_or(0)
    };

    // (post index, author, content)
    let comments: [(usize, i64, &str); 6] = [
        (0, alice_id, "沙发！社区看起来很棒。"),
        (0, bob_id, "支持，希望越办越好。"),
        (2, admin_id, "lambda DSL 让安全配置更清爽了。"),
        (4, alice_id, "照片拍得真好看！"),
        (6, alice_id, "已实名认证，可以正常发言。"),
        (8, admin_id, "很有个人风格，继续加油！"),
    ];
    for (idx, author_id, content) in comments.iter() {
        sqlx::query(
            "INSERT INTO comments (post_id, author_id, parent_id, content, upvotes, downvotes, score, is_removed, created_at, updated_at) \
             VALUES (?, ?, NULL, ?, 0, 0, 0, 0, NOW(), NOW())",
        )
        .bind(pid(*idx))
        .bind(author_id)
        .bind(content)
        .execute(&mut *tx)
        .await?;
    }

    // 回写评论数
    for (_idx, post_id) in created_post_ids.iter() {
        let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comments WHERE post_id = ? AND is_removed = 0")
            .bind(post_id)
            .fetch_one(&mut *tx)
            .await?;
        sqlx::query("UPDATE posts SET comment_count = ? WHERE id = ?")
            .bind(c as i32)
            .bind(post_id)
            .execute(&mut *tx)
            .await?;
    }

    // 回写帖子数
    for forum_id in [general, tech, life, adult, creative] {
        let (c,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM posts WHERE forum_id = ? AND is_removed = 0")
                .bind(forum_id)
                .fetch_one(&mut *tx)
                .await?;
        sqlx::query("UPDATE sub_forums SET post_count = ? WHERE id = ?")
            .bind(c as i32)
            .bind(forum_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    tracing::info!("Seeding completed: 3 users, 5 sub-forums.");
    Ok(())
}

async fn ensure_user(conn: &mut sqlx::MySqlConnection, u: &SeedUser) -> anyhow::Result<()> {
    let exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM users WHERE username = ? LIMIT 1")
        .bind(u.username)
        .fetch_optional(&mut *conn)
        .await?;
    if exists.is_some() {
        return Ok(());
    }
    let hash = hash_password(u.password)?;
    sqlx::query(
        "INSERT INTO users (username, email, password, google_id, avatar, bio, role, is_active, created_at, updated_at) \
         VALUES (?, ?, ?, NULL, NULL, ?, ?, 1, NOW(), NOW())",
    )
    .bind(u.username)
    .bind(u.email)
    .bind(&hash)
    .bind(u.bio)
    .bind(u.role)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

async fn user_id(conn: &mut sqlx::MySqlConnection, username: &str) -> anyhow::Result<i64> {
    let (id,): (i64,) = sqlx::query_as("SELECT id FROM users WHERE username = ?")
        .bind(username)
        .fetch_one(&mut *conn)
        .await?;
    Ok(id)
}

async fn seed_authentication(
    conn: &mut sqlx::MySqlConnection,
    username: &str,
    auth_type: &str,
    approved: bool,
    data: serde_json::Value,
) -> anyhow::Result<()> {
    let uid = user_id(&mut *conn, username).await?;
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM user_authentications WHERE user_id = ? AND type = ? LIMIT 1")
            .bind(uid)
            .bind(auth_type)
            .fetch_optional(&mut *conn)
            .await?;
    if exists.is_some() {
        return Ok(());
    }
    let (status, confidence, reason) = if approved {
        ("APPROVED", 0.95_f64, "规则校验通过")
    } else {
        ("PENDING", 0.5_f64, "等待人工审核")
    };
    let reviewed_at: Option<chrono::NaiveDateTime> = if approved {
        Some(chrono::Local::now().naive_local())
    } else {
        None
    };
    sqlx::query(
        "INSERT INTO user_authentications (user_id, type, auth_data, level, media, liveness, attestation, status, \
         review_method, confidence, reason, rejection_reason, reviewed_by, reviewed_at, created_at, updated_at) \
         VALUES (?, ?, ?, NULL, NULL, NULL, NULL, ?, 'RULE', ?, ?, NULL, NULL, ?, NOW(), NOW())",
    )
    .bind(uid)
    .bind(auth_type)
    .bind(data.to_string())
    .bind(status)
    .bind(confidence)
    .bind(reason)
    .bind(reviewed_at)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn create_forum(
    conn: &mut sqlx::MySqlConnection,
    slug: &str,
    name: &str,
    description: &str,
    theme_color: &str,
    required_auth_types: Option<&str>,
    owner_id: i64,
    members: &[(i64, &str)],
) -> anyhow::Result<i64> {
    let template_config = json!({
        "blocks": [{ "type": "announcement", "text": format!("{name} 欢迎你") }]
    })
    .to_string();
    let result = sqlx::query(
        "INSERT INTO sub_forums (slug, name, description, icon, theme_color, rules, required_auth_types, \
         template_type, template_config, custom_page_html, custom_page_status, custom_page_review_note, \
         creator_id, member_count, post_count, is_public, created_at, updated_at) \
         VALUES (?, ?, ?, '', ?, '请文明发言，遵守社区规范。', ?, 'DEFAULT', ?, NULL, NULL, NULL, ?, 0, 0, 1, NOW(), NOW())",
    )
    .bind(slug)
    .bind(name)
    .bind(description)
    .bind(theme_color)
    .bind(required_auth_types)
    .bind(&template_config)
    .bind(owner_id)
    .execute(&mut *conn)
    .await?;
    let forum_id = result.last_insert_id() as i64;

    sqlx::query(
        "INSERT INTO forum_members (forum_id, user_id, role, joined_at) VALUES (?, ?, 'OWNER', NOW())",
    )
    .bind(forum_id)
    .bind(owner_id)
    .execute(&mut *conn)
    .await?;
    for (uid, role) in members {
        sqlx::query(
            "INSERT INTO forum_members (forum_id, user_id, role, joined_at) VALUES (?, ?, ?, NOW())",
        )
        .bind(forum_id)
        .bind(uid)
        .bind(role)
        .execute(&mut *conn)
        .await?;
    }
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM forum_members WHERE forum_id = ?")
        .bind(forum_id)
        .fetch_one(&mut *conn)
        .await?;
    sqlx::query("UPDATE sub_forums SET member_count = ? WHERE id = ?")
        .bind(count as i32)
        .bind(forum_id)
        .execute(&mut *conn)
        .await?;
    Ok(forum_id)
}
