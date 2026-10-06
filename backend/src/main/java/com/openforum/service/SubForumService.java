package com.openforum.service;

import com.openforum.common.PageResponse;
import com.openforum.dto.forum.CustomPageDTO;
import com.openforum.dto.forum.ForumCreateRequest;
import com.openforum.dto.forum.ForumDetailDTO;
import com.openforum.dto.forum.ForumSummaryDTO;
import com.openforum.dto.forum.ForumUpdateRequest;
import com.openforum.dto.forum.MemberDTO;
import com.openforum.entity.ForumMember;
import com.openforum.entity.SubForum;
import com.openforum.entity.User;
import com.openforum.entity.enums.CustomPageStatus;
import com.openforum.entity.enums.ForumRole;
import com.openforum.entity.enums.TemplateType;
import com.openforum.exception.ApiException;
import com.openforum.plugin.PluginRegistry;
import com.openforum.plugin.RecommendationPlugin;
import com.openforum.repository.ForumMemberRepository;
import com.openforum.repository.SubForumRepository;
import com.openforum.repository.UserRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.List;
import java.util.Objects;
import java.util.Set;
import java.util.regex.Pattern;
import java.util.stream.Collectors;

@Service
public class SubForumService {

    private static final Pattern SLUG_PATTERN = Pattern.compile("^[a-z0-9_-]{2,80}$");

    private final SubForumRepository subForumRepository;
    private final ForumMemberRepository memberRepository;
    private final UserRepository userRepository;
    private final PluginRegistry pluginRegistry;
    private final AuthGateService gateService;
    private final ForumAccessService accessService;
    private final HtmlSanitizer htmlSanitizer;

    public SubForumService(SubForumRepository subForumRepository, ForumMemberRepository memberRepository,
                           UserRepository userRepository, PluginRegistry pluginRegistry,
                           AuthGateService gateService, ForumAccessService accessService,
                           HtmlSanitizer htmlSanitizer) {
        this.subForumRepository = subForumRepository;
        this.memberRepository = memberRepository;
        this.userRepository = userRepository;
        this.pluginRegistry = pluginRegistry;
        this.gateService = gateService;
        this.accessService = accessService;
        this.htmlSanitizer = htmlSanitizer;
    }

    public PageResponse<ForumSummaryDTO> list(int page, int size, String sort, String query, Long userId) {
        List<SubForum> forums = (query == null || query.isBlank())
                ? new ArrayList<>(subForumRepository.findAll())
                : new ArrayList<>(subForumRepository.findByNameContainingIgnoreCaseOrSlugContainingIgnoreCase(query, query));
        Comparator<SubForum> comparator = "new".equalsIgnoreCase(sort)
                ? Comparator.comparing(SubForum::getCreatedAt).reversed()
                : Comparator.<SubForum>comparingDouble(this::pluginScore).reversed();
        forums.sort(comparator);

        Set<Long> memberIds = accessService.memberForumIds(userId);
        int total = forums.size();
        int from = Math.max(0, Math.min(page * size, total));
        int to = Math.min(from + size, total);
        List<ForumSummaryDTO> content = forums.subList(from, to).stream()
                .map(forum -> toSummary(forum, memberIds.contains(forum.getId())))
                .toList();
        return PageResponse.of(content, page, size, total);
    }

    public List<ForumSummaryDTO> popular(int limit) {
        List<SubForum> forums = new ArrayList<>(subForumRepository.findAll());
        forums.sort(Comparator.comparingInt(SubForum::getMemberCount).reversed()
                .thenComparing(Comparator.comparingInt(SubForum::getPostCount).reversed()));
        return forums.stream().limit(limit).map(forum -> toSummary(forum, false)).toList();
    }

    public List<ForumSummaryDTO> recommended(int limit, Long userId) {
        List<SubForum> forums = new ArrayList<>(subForumRepository.findAll());
        Set<Long> memberIds = accessService.memberForumIds(userId);
        if (userId == null) {
            forums.sort(Comparator.<SubForum>comparingDouble(this::pluginScore).reversed());
        } else {
            forums.sort(Comparator.<SubForum>comparingDouble(
                    forum -> pluginScore(forum) + (memberIds.contains(forum.getId()) ? 1000 : 0)).reversed());
        }
        return forums.stream().limit(limit)
                .map(forum -> toSummary(forum, memberIds.contains(forum.getId())))
                .toList();
    }

    public ForumDetailDTO detail(String slug, Long userId) {
        SubForum forum = subForumRepository.findBySlug(slug)
                .orElseThrow(() -> ApiException.notFound("子论坛不存在"));
        return toDetail(forum, userId);
    }

    @Transactional
    public ForumDetailDTO create(ForumCreateRequest request, Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        String slug = request.slug() == null ? "" : request.slug().trim().toLowerCase();
        if (!SLUG_PATTERN.matcher(slug).matches()) {
            throw ApiException.badRequest("slug 只能包含小写字母、数字、-、_（2-80 位）");
        }
        if (subForumRepository.existsBySlug(slug)) {
            throw ApiException.badRequest("slug 已存在");
        }
        SubForum forum = new SubForum();
        forum.setSlug(slug);
        forum.setName(request.name());
        forum.setDescription(request.description());
        forum.setIcon(request.icon());
        forum.setThemeColor(request.themeColor() == null ? "#FF4500" : request.themeColor());
        forum.setRules(request.rules());
        forum.setRequiredAuthTypes(joinAuthTypes(request.requiredAuthTypes()));
        forum.setTemplateType(parseTemplateType(request.templateType()));
        forum.setTemplateConfig(request.templateConfig());
        forum.setCreatorId(userId);
        forum.setPublic(true);
        subForumRepository.save(forum);

        ForumMember member = new ForumMember();
        member.setForumId(forum.getId());
        member.setUserId(userId);
        member.setRole(ForumRole.OWNER);
        memberRepository.save(member);
        forum.setMemberCount(1);
        subForumRepository.save(forum);
        return toDetail(forum, userId);
    }

    @Transactional
    public ForumDetailDTO update(Long forumId, ForumUpdateRequest request, Long userId) {
        SubForum forum = require(forumId);
        assertManage(forum, userId);
        if (request.name() != null) {
            forum.setName(request.name());
        }
        if (request.description() != null) {
            forum.setDescription(request.description());
        }
        if (request.icon() != null) {
            forum.setIcon(request.icon());
        }
        if (request.themeColor() != null) {
            forum.setThemeColor(request.themeColor());
        }
        if (request.rules() != null) {
            forum.setRules(request.rules());
        }
        if (request.requiredAuthTypes() != null) {
            forum.setRequiredAuthTypes(joinAuthTypes(request.requiredAuthTypes()));
        }
        if (request.templateType() != null) {
            forum.setTemplateType(parseTemplateType(request.templateType()));
        }
        if (request.templateConfig() != null) {
            forum.setTemplateConfig(request.templateConfig());
        }
        subForumRepository.save(forum);
        return toDetail(forum, userId);
    }

    @Transactional
    public CustomPageDTO saveCustomPage(Long forumId, String html, Long userId) {
        SubForum forum = require(forumId);
        assertManage(forum, userId);
        if (html == null || html.isBlank()) {
            throw ApiException.badRequest("html 不能为空");
        }
        forum.setCustomPageHtml(htmlSanitizer.sanitize(html));
        forum.setCustomPageStatus(CustomPageStatus.DRAFT);
        forum.setCustomPageReviewNote(null);
        subForumRepository.save(forum);
        return toCustomPage(forum);
    }

    @Transactional
    public CustomPageDTO submitCustomPage(Long forumId, Long userId) {
        SubForum forum = require(forumId);
        assertManage(forum, userId);
        if (forum.getCustomPageHtml() == null || forum.getCustomPageHtml().isBlank()) {
            throw ApiException.badRequest("请先保存自定义页面 HTML");
        }
        forum.setCustomPageStatus(CustomPageStatus.PENDING);
        forum.setCustomPageReviewNote(null);
        subForumRepository.save(forum);
        return toCustomPage(forum);
    }

    public CustomPageDTO getCustomPage(Long forumId, Long userId) {
        SubForum forum = require(forumId);
        if (!accessService.canManage(forum, userId)) {
            throw ApiException.forbidden("需要坛主或管理员权限");
        }
        return toCustomPage(forum);
    }

    public PageResponse<CustomPageDTO> listCustomPages(String status, int page, int size) {
        CustomPageStatus pageStatus = parseCustomPageStatus(status);
        List<SubForum> forums = new ArrayList<>(
                subForumRepository.findByCustomPageStatusOrderByUpdatedAtDesc(pageStatus));
        int total = forums.size();
        int from = Math.max(0, Math.min(page * size, total));
        int to = Math.min(from + size, total);
        List<CustomPageDTO> content = forums.subList(from, to).stream().map(this::toCustomPage).toList();
        return PageResponse.of(content, page, size, total);
    }

    @Transactional
    public CustomPageDTO approveCustomPage(Long forumId) {
        SubForum forum = require(forumId);
        forum.setCustomPageStatus(CustomPageStatus.APPROVED);
        forum.setCustomPageReviewNote(null);
        forum.setTemplateType(TemplateType.CUSTOM);
        subForumRepository.save(forum);
        return toCustomPage(forum);
    }

    @Transactional
    public CustomPageDTO rejectCustomPage(Long forumId, String reason) {
        SubForum forum = require(forumId);
        forum.setCustomPageStatus(CustomPageStatus.REJECTED);
        forum.setCustomPageReviewNote(reason == null || reason.isBlank() ? "管理员驳回" : reason);
        subForumRepository.save(forum);
        return toCustomPage(forum);
    }

    @Transactional
    public void join(Long forumId, Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        SubForum forum = require(forumId);
        if (!memberRepository.existsByForumIdAndUserId(forumId, userId)) {
            ForumMember member = new ForumMember();
            member.setForumId(forumId);
            member.setUserId(userId);
            member.setRole(ForumRole.MEMBER);
            memberRepository.save(member);
        }
        forum.setMemberCount((int) memberRepository.countByForumId(forumId));
        subForumRepository.save(forum);
    }

    @Transactional
    public void leave(Long forumId, Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        SubForum forum = require(forumId);
        if (userId.equals(forum.getCreatorId())) {
            throw ApiException.badRequest("坛主不能退出自己的子论坛");
        }
        memberRepository.deleteByForumIdAndUserId(forumId, userId);
        forum.setMemberCount((int) memberRepository.countByForumId(forumId));
        subForumRepository.save(forum);
    }

    public List<MemberDTO> members(Long forumId) {
        require(forumId);
        return memberRepository.findByForumIdOrderByJoinedAtAsc(forumId).stream()
                .map(member -> {
                    User user = userRepository.findById(member.getUserId()).orElse(null);
                    return new MemberDTO(member.getUserId(),
                            user == null ? null : user.getUsername(),
                            user == null ? null : user.getAvatar(),
                            member.getRole().name(),
                            member.getJoinedAt());
                })
                .toList();
    }

    public SubForum require(Long forumId) {
        return subForumRepository.findById(forumId)
                .orElseThrow(() -> ApiException.notFound("子论坛不存在"));
    }

    private void assertManage(SubForum forum, Long userId) {
        if (!accessService.canManage(forum, userId)) {
            throw ApiException.forbidden("需要坛主或管理员权限");
        }
    }

    private ForumSummaryDTO toSummary(SubForum forum, boolean isMember) {
        return new ForumSummaryDTO(forum.getId(), forum.getSlug(), forum.getName(), forum.getDescription(),
                forum.getIcon(), forum.getThemeColor(), authTypeList(forum.getRequiredAuthTypes()),
                forum.getMemberCount(), forum.getPostCount(), isMember, forum.getCreatedAt());
    }

    private ForumDetailDTO toDetail(SubForum forum, Long userId) {
        boolean isMember = accessService.isMember(forum.getId(), userId);
        boolean canPost = userId != null && gateService.evaluate(userId, forum.getRequiredAuthTypes()).passed();
        String customHtml = forum.getCustomPageStatus() == CustomPageStatus.APPROVED ? forum.getCustomPageHtml() : null;
        String creatorName = userRepository.findById(forum.getCreatorId()).map(User::getUsername).orElse(null);
        return new ForumDetailDTO(forum.getId(), forum.getSlug(), forum.getName(), forum.getDescription(),
                forum.getIcon(), forum.getThemeColor(), forum.getRules(),
                authTypeList(forum.getRequiredAuthTypes()),
                forum.getTemplateType() == null ? TemplateType.DEFAULT.name() : forum.getTemplateType().name(),
                forum.getTemplateConfig(),
                forum.getCustomPageStatus() == null ? null : forum.getCustomPageStatus().name(),
                customHtml, forum.getMemberCount(), forum.getPostCount(), isMember, canPost, forum.isPublic(),
                forum.getCreatorId(), creatorName, forum.getCreatedAt());
    }

    private CustomPageDTO toCustomPage(SubForum forum) {
        return new CustomPageDTO(forum.getId(), forum.getSlug(), forum.getName(), forum.getCustomPageHtml(),
                forum.getCustomPageStatus() == null ? null : forum.getCustomPageStatus().name(),
                forum.getCustomPageReviewNote());
    }

    private double pluginScore(SubForum forum) {
        double total = 0;
        for (RecommendationPlugin plugin : pluginRegistry.enabledRecommendationPlugins()) {
            total += plugin.score(forum);
        }
        if (total <= 0) {
            total = forum.getMemberCount() * 2.0 + forum.getPostCount() * 3.0;
        }
        return total;
    }

    private List<String> authTypeList(String csv) {
        if (csv == null || csv.isBlank()) {
            return List.of();
        }
        return Arrays.stream(csv.split(","))
                .map(String::trim)
                .filter(value -> !value.isEmpty())
                .toList();
    }

    private String joinAuthTypes(List<String> types) {
        if (types == null || types.isEmpty()) {
            return null;
        }
        return types.stream()
                .filter(Objects::nonNull)
                .map(value -> value.trim().toUpperCase())
                .filter(value -> !value.isEmpty())
                .distinct()
                .collect(Collectors.joining(","));
    }

    private TemplateType parseTemplateType(String value) {
        if (value == null || value.isBlank()) {
            return TemplateType.DEFAULT;
        }
        try {
            return TemplateType.valueOf(value.trim().toUpperCase());
        } catch (IllegalArgumentException ex) {
            return TemplateType.DEFAULT;
        }
    }

    private CustomPageStatus parseCustomPageStatus(String value) {
        if (value == null || value.isBlank()) {
            return CustomPageStatus.PENDING;
        }
        try {
            return CustomPageStatus.valueOf(value.trim().toUpperCase());
        } catch (IllegalArgumentException ex) {
            throw ApiException.badRequest("非法的状态: " + value);
        }
    }
}
