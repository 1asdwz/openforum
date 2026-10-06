package com.openforum.controller;

import com.openforum.common.ApiResponse;
import com.openforum.dto.comment.CommentCreateRequest;
import com.openforum.dto.comment.CommentDTO;
import com.openforum.dto.common.VoteRequest;
import com.openforum.security.SecurityUtils;
import com.openforum.service.CommentService;
import jakarta.validation.Valid;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;

@RestController
@RequestMapping("/comments")
public class CommentController {

    private final CommentService commentService;

    public CommentController(CommentService commentService) {
        this.commentService = commentService;
    }

    @GetMapping("/post/{postId}")
    public ApiResponse<List<CommentDTO>> tree(@PathVariable Long postId) {
        return ApiResponse.ok(commentService.tree(postId));
    }

    @PostMapping
    public ApiResponse<CommentDTO> create(@Valid @RequestBody CommentCreateRequest request) {
        return ApiResponse.ok(commentService.create(request, SecurityUtils.requireUser().id()));
    }

    @DeleteMapping("/{id}")
    public ApiResponse<Void> delete(@PathVariable Long id) {
        commentService.delete(id, SecurityUtils.requireUser().id());
        return ApiResponse.ok();
    }

    @PostMapping("/{id}/vote")
    public ApiResponse<CommentDTO> vote(@PathVariable Long id, @RequestBody VoteRequest request) {
        return ApiResponse.ok(commentService.vote(id, request.value(), SecurityUtils.requireUser().id()));
    }
}
