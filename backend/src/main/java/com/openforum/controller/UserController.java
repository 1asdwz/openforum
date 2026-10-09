package com.openforum.controller;

import com.openforum.common.ApiResponse;
import com.openforum.common.PageResponse;
import com.openforum.dto.comment.CommentDTO;
import com.openforum.dto.post.PostSummaryDTO;
import com.openforum.dto.user.UpdateProfileRequest;
import com.openforum.dto.user.UserDTO;
import com.openforum.security.SecurityUtils;
import com.openforum.service.CommentService;
import com.openforum.service.PostService;
import com.openforum.service.UserService;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/users")
public class UserController {

    private final UserService userService;
    private final PostService postService;
    private final CommentService commentService;

    public UserController(UserService userService, PostService postService, CommentService commentService) {
        this.userService = userService;
        this.postService = postService;
        this.commentService = commentService;
    }

    @GetMapping("/{id}")
    public ApiResponse<UserDTO> profile(@PathVariable Long id) {
        return ApiResponse.ok(userService.getPublicProfile(id));
    }

    @GetMapping("/{id}/posts")
    public ApiResponse<PageResponse<PostSummaryDTO>> posts(
            @PathVariable Long id,
            @RequestParam(defaultValue = "new") String sort,
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size) {
        userService.require(id);
        return ApiResponse.ok(postService.listByAuthor(id, sort, page, size));
    }

    @GetMapping("/{id}/comments")
    public ApiResponse<PageResponse<CommentDTO>> comments(
            @PathVariable Long id,
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size) {
        userService.require(id);
        return ApiResponse.ok(commentService.listByAuthor(id, page, size));
    }

    @PutMapping("/me")
    public ApiResponse<UserDTO> updateMe(@RequestBody UpdateProfileRequest request) {
        return ApiResponse.ok(userService.updateMe(SecurityUtils.requireUser().id(), request));
    }
}
