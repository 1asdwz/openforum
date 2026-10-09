package com.openforum.controller;

import com.openforum.common.ApiResponse;
import com.openforum.common.PageResponse;
import com.openforum.dto.common.VoteRequest;
import com.openforum.dto.post.PostCreateRequest;
import com.openforum.dto.post.PostDetailDTO;
import com.openforum.dto.post.PostSummaryDTO;
import com.openforum.dto.post.PostUpdateRequest;
import com.openforum.security.AuthUser;
import com.openforum.security.SecurityUtils;
import com.openforum.service.PostService;
import jakarta.validation.Valid;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/posts")
public class PostController {

    private final PostService postService;

    public PostController(PostService postService) {
        this.postService = postService;
    }

    @GetMapping
    public ApiResponse<PageResponse<PostSummaryDTO>> list(
            @RequestParam(required = false) Long forumId,
            @RequestParam(required = false) String q,
            @RequestParam(defaultValue = "hot") String sort,
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size) {
        return ApiResponse.ok(postService.list(forumId, q, sort, page, size, currentUserId()));
    }

    @GetMapping("/{id}")
    public ApiResponse<PostDetailDTO> detail(@PathVariable Long id) {
        return ApiResponse.ok(postService.detail(id, currentUserId()));
    }

    @PostMapping
    public ApiResponse<PostDetailDTO> create(@Valid @RequestBody PostCreateRequest request) {
        return ApiResponse.ok(postService.create(request, SecurityUtils.requireUser().id()));
    }

    @PutMapping("/{id}")
    public ApiResponse<PostDetailDTO> update(@PathVariable Long id, @RequestBody PostUpdateRequest request) {
        return ApiResponse.ok(postService.update(id, request, SecurityUtils.requireUser().id()));
    }

    @DeleteMapping("/{id}")
    public ApiResponse<Void> delete(@PathVariable Long id) {
        postService.delete(id, SecurityUtils.requireUser().id());
        return ApiResponse.ok();
    }

    @PostMapping("/{id}/vote")
    public ApiResponse<PostDetailDTO> vote(@PathVariable Long id, @RequestBody VoteRequest request) {
        return ApiResponse.ok(postService.vote(id, request.value(), SecurityUtils.requireUser().id()));
    }

    @PostMapping("/{id}/pin")
    public ApiResponse<PostDetailDTO> pin(@PathVariable Long id) {
        return ApiResponse.ok(postService.pin(id, SecurityUtils.requireUser().id()));
    }

    private Long currentUserId() {
        return SecurityUtils.currentUser().map(AuthUser::id).orElse(null);
    }
}
