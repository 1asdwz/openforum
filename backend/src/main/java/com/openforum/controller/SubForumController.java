package com.openforum.controller;

import com.openforum.common.ApiResponse;
import com.openforum.common.PageResponse;
import com.openforum.dto.forum.CustomPageDTO;
import com.openforum.dto.forum.CustomPageUpdateRequest;
import com.openforum.dto.forum.ForumCreateRequest;
import com.openforum.dto.forum.ForumDetailDTO;
import com.openforum.dto.forum.ForumSummaryDTO;
import com.openforum.dto.forum.ForumUpdateRequest;
import com.openforum.dto.forum.MemberDTO;
import com.openforum.security.AuthUser;
import com.openforum.security.SecurityUtils;
import com.openforum.service.SubForumService;
import jakarta.validation.Valid;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;

@RestController
@RequestMapping("/forums")
public class SubForumController {

    private final SubForumService subForumService;

    public SubForumController(SubForumService subForumService) {
        this.subForumService = subForumService;
    }

    @GetMapping
    public ApiResponse<PageResponse<ForumSummaryDTO>> list(
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size,
            @RequestParam(defaultValue = "hot") String sort,
            @RequestParam(required = false) String q) {
        return ApiResponse.ok(subForumService.list(page, size, sort, q, currentUserId()));
    }

    @GetMapping("/popular")
    public ApiResponse<List<ForumSummaryDTO>> popular(@RequestParam(defaultValue = "10") int limit) {
        return ApiResponse.ok(subForumService.popular(limit));
    }

    @GetMapping("/recommended")
    public ApiResponse<List<ForumSummaryDTO>> recommended(@RequestParam(defaultValue = "10") int limit) {
        return ApiResponse.ok(subForumService.recommended(limit, currentUserId()));
    }

    @PostMapping
    public ApiResponse<ForumDetailDTO> create(@Valid @RequestBody ForumCreateRequest request) {
        return ApiResponse.ok(subForumService.create(request, SecurityUtils.requireUser().id()));
    }

    @GetMapping("/{slug}")
    public ApiResponse<ForumDetailDTO> detail(@PathVariable String slug) {
        return ApiResponse.ok(subForumService.detail(slug, currentUserId()));
    }

    @PutMapping("/{id}")
    public ApiResponse<ForumDetailDTO> update(@PathVariable Long id,
                                              @RequestBody ForumUpdateRequest request) {
        return ApiResponse.ok(subForumService.update(id, request, SecurityUtils.requireUser().id()));
    }

    @PutMapping("/{id}/custom-page")
    public ApiResponse<CustomPageDTO> saveCustomPage(@PathVariable Long id,
                                                     @Valid @RequestBody CustomPageUpdateRequest request) {
        return ApiResponse.ok(subForumService.saveCustomPage(id, request.html(), SecurityUtils.requireUser().id()));
    }

    @PostMapping("/{id}/custom-page/submit")
    public ApiResponse<CustomPageDTO> submitCustomPage(@PathVariable Long id) {
        return ApiResponse.ok(subForumService.submitCustomPage(id, SecurityUtils.requireUser().id()));
    }

    @GetMapping("/{id}/custom-page")
    public ApiResponse<CustomPageDTO> getCustomPage(@PathVariable Long id) {
        return ApiResponse.ok(subForumService.getCustomPage(id, SecurityUtils.requireUser().id()));
    }

    @PostMapping("/{id}/join")
    public ApiResponse<Void> join(@PathVariable Long id) {
        subForumService.join(id, SecurityUtils.requireUser().id());
        return ApiResponse.ok();
    }

    @PostMapping("/{id}/leave")
    public ApiResponse<Void> leave(@PathVariable Long id) {
        subForumService.leave(id, SecurityUtils.requireUser().id());
        return ApiResponse.ok();
    }

    @GetMapping("/{id}/members")
    public ApiResponse<List<MemberDTO>> members(@PathVariable Long id) {
        return ApiResponse.ok(subForumService.members(id));
    }

    private Long currentUserId() {
        return SecurityUtils.currentUser().map(AuthUser::id).orElse(null);
    }
}
