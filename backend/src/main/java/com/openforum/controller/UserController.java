package com.openforum.controller;

import com.openforum.common.ApiResponse;
import com.openforum.dto.user.UpdateProfileRequest;
import com.openforum.dto.user.UserDTO;
import com.openforum.security.SecurityUtils;
import com.openforum.service.UserService;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/users")
public class UserController {

    private final UserService userService;

    public UserController(UserService userService) {
        this.userService = userService;
    }

    @GetMapping("/{id}")
    public ApiResponse<UserDTO> profile(@PathVariable Long id) {
        return ApiResponse.ok(userService.getPublicProfile(id));
    }

    @PutMapping("/me")
    public ApiResponse<UserDTO> updateMe(@RequestBody UpdateProfileRequest request) {
        return ApiResponse.ok(userService.updateMe(SecurityUtils.requireUser().id(), request));
    }
}
