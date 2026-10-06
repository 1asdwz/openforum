package com.openforum.controller;

import com.openforum.common.ApiResponse;
import com.openforum.common.PageResponse;
import com.openforum.dto.authentication.AuthenticationDTO;
import com.openforum.dto.common.ReviewDecisionRequest;
import com.openforum.dto.forum.CustomPageDTO;
import com.openforum.dto.user.UserDTO;
import com.openforum.security.AuthUser;
import com.openforum.security.SecurityUtils;
import com.openforum.service.AuthenticationService;
import com.openforum.service.SubForumService;
import com.openforum.service.UserService;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/admin")
public class AdminController {

    private final AuthenticationService authenticationService;
    private final SubForumService subForumService;
    private final UserService userService;

    public AdminController(AuthenticationService authenticationService, SubForumService subForumService,
                           UserService userService) {
        this.authenticationService = authenticationService;
        this.subForumService = subForumService;
        this.userService = userService;
    }

    @GetMapping("/authentications")
    public ApiResponse<PageResponse<AuthenticationDTO>> authentications(
            @RequestParam(defaultValue = "PENDING") String status,
            @RequestParam(required = false) String level,
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size) {
        SecurityUtils.requireAdmin();
        return ApiResponse.ok(authenticationService.adminList(status, level, page, size));
    }

    @PostMapping("/authentications/{id}/approve")
    public ApiResponse<AuthenticationDTO> approveAuthentication(
            @PathVariable Long id,
            @RequestBody(required = false) ReviewDecisionRequest request) {
        AuthUser admin = SecurityUtils.requireAdmin();
        return ApiResponse.ok(authenticationService.approve(id, reason(request), admin.id()));
    }

    @PostMapping("/authentications/{id}/reject")
    public ApiResponse<AuthenticationDTO> rejectAuthentication(
            @PathVariable Long id,
            @RequestBody(required = false) ReviewDecisionRequest request) {
        AuthUser admin = SecurityUtils.requireAdmin();
        return ApiResponse.ok(authenticationService.reject(id, reason(request), admin.id()));
    }

    @GetMapping("/custom-pages")
    public ApiResponse<PageResponse<CustomPageDTO>> customPages(
            @RequestParam(defaultValue = "PENDING") String status,
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size) {
        SecurityUtils.requireAdmin();
        return ApiResponse.ok(subForumService.listCustomPages(status, page, size));
    }

    @PostMapping("/custom-pages/{id}/approve")
    public ApiResponse<CustomPageDTO> approveCustomPage(@PathVariable Long id) {
        SecurityUtils.requireAdmin();
        return ApiResponse.ok(subForumService.approveCustomPage(id));
    }

    @PostMapping("/custom-pages/{id}/reject")
    public ApiResponse<CustomPageDTO> rejectCustomPage(
            @PathVariable Long id,
            @RequestBody(required = false) ReviewDecisionRequest request) {
        SecurityUtils.requireAdmin();
        return ApiResponse.ok(subForumService.rejectCustomPage(id, reason(request)));
    }

    @GetMapping("/users")
    public ApiResponse<PageResponse<UserDTO>> users(
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size) {
        SecurityUtils.requireAdmin();
        return ApiResponse.ok(userService.listUsers(page, size));
    }

    private String reason(ReviewDecisionRequest request) {
        return request == null ? null : request.reason();
    }
}
