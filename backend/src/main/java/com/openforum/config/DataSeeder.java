package com.openforum.config;

import com.openforum.common.JsonSupport;
import com.openforum.entity.Comment;
import com.openforum.entity.ForumMember;
import com.openforum.entity.Post;
import com.openforum.entity.SubForum;
import com.openforum.entity.User;
import com.openforum.entity.UserAuthentication;
import com.openforum.entity.enums.AuthStatus;
import com.openforum.entity.enums.AuthType;
import com.openforum.entity.enums.ForumRole;
import com.openforum.entity.enums.ReviewMethod;
import com.openforum.entity.enums.TemplateType;
import com.openforum.entity.enums.UserRole;
import com.openforum.repository.CommentRepository;
import com.openforum.repository.ForumMemberRepository;
import com.openforum.repository.PostRepository;
import com.openforum.repository.SubForumRepository;
import com.openforum.repository.UserAuthenticationRepository;
import com.openforum.repository.UserRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.ApplicationArguments;
import org.springframework.boot.ApplicationRunner;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Component;
import org.springframework.transaction.annotation.Transactional;

import java.time.LocalDateTime;
import java.util.Map;

/**
 * Idempotent seed data: admin/admin123, alice/alice123, bob/bob123 and five demo sub-forums
 * with posts and comments. Runs once (skipped when sub-forums already exist).
 */
@Component
public class DataSeeder implements ApplicationRunner {

    private static final Logger log = LoggerFactory.getLogger(DataSeeder.class);

    private final UserRepository userRepository;
    private final SubForumRepository subForumRepository;
    private final ForumMemberRepository memberRepository;
    private final PostRepository postRepository;
    private final CommentRepository commentRepository;
    private final UserAuthenticationRepository authenticationRepository;
    private final PasswordEncoder passwordEncoder;
    private final JsonSupport jsonSupport;

    public DataSeeder(UserRepository userRepository, SubForumRepository subForumRepository,
                      ForumMemberRepository memberRepository, PostRepository postRepository,
                      CommentRepository commentRepository, UserAuthenticationRepository authenticationRepository,
                      PasswordEncoder passwordEncoder, JsonSupport jsonSupport) {
        this.userRepository = userRepository;
        this.subForumRepository = subForumRepository;
        this.memberRepository = memberRepository;
        this.postRepository = postRepository;
        this.commentRepository = commentRepository;
        this.authenticationRepository = authenticationRepository;
        this.passwordEncoder = passwordEncoder;
        this.jsonSupport = jsonSupport;
    }

    @Override
    @Transactional
    public void run(ApplicationArguments args) {
        if (subForumRepository.count() > 0) {
            return;
        }
        log.info("Seeding OpenForum demo data...");

        User admin = ensureUser("admin", "admin@openforum.local", "admin123", UserRole.ADMIN, "OpenForum 站点管理员");
        User alice = ensureUser("alice", "alice@openforum.local", "alice123", UserRole.USER, "热爱技术与生活的普通用户");
        User bob = ensureUser("bob", "bob@openforum.local", "bob123", UserRole.USER, "潜水多年的老用户");

        seedAuthentication(alice, AuthType.REAL_NAME, Map.of("fullName", "爱丽丝"), true);
        seedAuthentication(alice, AuthType.AGE, Map.of("age", 25), true);
        seedAuthentication(bob, AuthType.REAL_NAME, Map.of("fullName", "鲍勃"), true);
        seedAuthentication(bob, AuthType.NATIONALITY, Map.of("nationality", "中国"), false);

        SubForum general = createForum("general", "综合讨论区", "任何话题都可以在这里畅聊",
                "", "#FF4500", null, admin, alice, bob);
        SubForum tech = createForum("tech", "技术交流", "编程、架构与开源技术分享",
                "", "#3498DB", null, admin, alice);
        SubForum life = createForum("life", "生活百态", "记录生活的点点滴滴",
                "", "#2ECC71", null, admin, bob);
        SubForum adult = createForum("adult", "成人话题", "仅限实名且年满 18 周岁的用户参与",
                "", "#9B59B6", "REAL_NAME,AGE_18", admin, alice);
        SubForum creative = createForum("creative", "创意工坊", "展示你的创作与灵感",
                "", "#E67E22", null, admin, alice);

        Post generalWelcome = createPost(general, admin, "欢迎来到 OpenForum！",
                "这里是开放的社区，欢迎分享你的想法与见解。", true);
        Post generalIntro = createPost(general, alice, "新人报到，请多关照",
                "大家好，我是爱丽丝，很高兴认识大家！", false);
        addComment(generalWelcome, alice, "沙发！社区看起来很棒。");
        addComment(generalWelcome, bob, "支持，希望越办越好。");

        Post techSpring = createPost(tech, alice, "Spring Boot 4 有哪些值得关注的变化？",
                "Java 25 + Spring Boot 4 的组合让人期待，欢迎一起讨论。", true);
        Post techMysql = createPost(tech, admin, "MySQL 索引优化的一些实践经验",
                "合理使用联合索引能显著提升查询性能，这里分享几个真实案例。", false);
        addComment(techSpring, admin, "lambda DSL 让安全配置更清爽了。");

        Post lifeTravel = createPost(life, bob, "周末去了趟郊外，风景很好",
                "远离城市喧嚣，呼吸新鲜空气，推荐大家也去走走。", false);
        Post lifeCoffee = createPost(life, bob, "分享一家宝藏咖啡馆",
                "手冲咖啡香气浓郁，环境也很安静，适合阅读。", false);
        addComment(lifeTravel, alice, "照片拍得真好看！");

        Post adultDiscuss = createPost(adult, admin, "成人话题区使用须知",
                "本板块仅对完成实名认证且年龄满 18 周岁的用户开放。", true);
        Post adultLife = createPost(adult, alice, "成年人的理财心得",
                "理性消费、长期投资，是我一直坚持的原则。", false);
        addComment(adultDiscuss, alice, "已实名认证，可以正常发言。");

        Post creativeArt = createPost(creative, alice, "分享我的插画练习作品",
                "最近在练习人物插画，欢迎大家指点。", true);
        Post creativeStory = createPost(creative, alice, "写了一篇短篇科幻小说",
                "关于时间旅行的一个小故事，欢迎阅读。", false);
        addComment(creativeArt, admin, "很有个人风格，继续加油！");

        for (SubForum forum : new SubForum[]{general, tech, life, adult, creative}) {
            forum.setPostCount((int) postRepository.countByForumIdAndRemovedFalse(forum.getId()));
            subForumRepository.save(forum);
        }
        log.info("Seeding completed: 3 users, 5 sub-forums.");
    }

    private User ensureUser(String username, String email, String rawPassword, UserRole role, String bio) {
        return userRepository.findByUsername(username).orElseGet(() -> {
            User user = new User();
            user.setUsername(username);
            user.setEmail(email);
            user.setPassword(passwordEncoder.encode(rawPassword));
            user.setRole(role);
            user.setActive(true);
            user.setBio(bio);
            return userRepository.save(user);
        });
    }

    private void seedAuthentication(User user, AuthType type, Map<String, Object> data, boolean approved) {
        if (authenticationRepository.findByUserIdAndType(user.getId(), type).isPresent()) {
            return;
        }
        UserAuthentication auth = new UserAuthentication();
        auth.setUserId(user.getId());
        auth.setType(type);
        auth.setAuthData(jsonSupport.toJson(data));
        auth.setStatus(approved ? AuthStatus.APPROVED : AuthStatus.PENDING);
        auth.setReviewMethod(ReviewMethod.RULE);
        auth.setConfidence(approved ? 0.95 : 0.5);
        auth.setReason(approved ? "规则校验通过" : "等待人工审核");
        if (approved) {
            auth.setReviewedAt(LocalDateTime.now());
        }
        authenticationRepository.save(auth);
    }

    private SubForum createForum(String slug, String name, String description, String icon, String themeColor,
                                 String requiredAuthTypes, User owner, User... members) {
        SubForum forum = new SubForum();
        forum.setSlug(slug);
        forum.setName(name);
        forum.setDescription(description);
        forum.setIcon(icon);
        forum.setThemeColor(themeColor);
        forum.setRules("请文明发言，遵守社区规范。");
        forum.setRequiredAuthTypes(requiredAuthTypes);
        forum.setTemplateType(TemplateType.DEFAULT);
        forum.setTemplateConfig("{\"blocks\":[{\"type\":\"announcement\",\"text\":\"" + name + " 欢迎你\"}]}");
        forum.setCreatorId(owner.getId());
        forum.setPublic(true);
        subForumRepository.save(forum);

        addMember(forum.getId(), owner.getId(), ForumRole.OWNER);
        for (User member : members) {
            addMember(forum.getId(), member.getId(), ForumRole.MEMBER);
        }
        forum.setMemberCount((int) memberRepository.countByForumId(forum.getId()));
        subForumRepository.save(forum);
        return forum;
    }

    private void addMember(Long forumId, Long userId, ForumRole role) {
        if (memberRepository.existsByForumIdAndUserId(forumId, userId)) {
            return;
        }
        ForumMember member = new ForumMember();
        member.setForumId(forumId);
        member.setUserId(userId);
        member.setRole(role);
        memberRepository.save(member);
    }

    private Post createPost(SubForum forum, User author, String title, String content, boolean pinned) {
        Post post = new Post();
        post.setForumId(forum.getId());
        post.setAuthorId(author.getId());
        post.setTitle(title);
        post.setContent(content);
        post.setPinned(pinned);
        return postRepository.save(post);
    }

    private void addComment(Post post, User author, String content) {
        Comment comment = new Comment();
        comment.setPostId(post.getId());
        comment.setAuthorId(author.getId());
        comment.setContent(content);
        commentRepository.save(comment);
        post.setCommentCount((int) commentRepository.countByPostIdAndRemovedFalse(post.getId()));
        postRepository.save(post);
    }
}
