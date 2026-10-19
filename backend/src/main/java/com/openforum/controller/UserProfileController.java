package com.openforum.controller;

import com.openforum.common.ApiResponse;
import com.openforum.dto.user.UpdateUserProfileRequest;
import com.openforum.dto.user.UserProfileDTO;
import com.openforum.security.SecurityUtils;
import com.openforum.service.UserProfileService;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/users")
public class UserProfileController {

    private final UserProfileService profileService;

    public UserProfileController(UserProfileService profileService) {
        this.profileService = profileService;
    }

    /** 获取当前用户的扩展档案 */
    @GetMapping("/me/profile")
    public ApiResponse<UserProfileDTO> getMyProfile() {
        return ApiResponse.ok(profileService.getProfile(SecurityUtils.requireUser().id()));
    }

    /** 更新当前用户的扩展档案（增量更新，null 字段不覆盖） */
    @PutMapping("/me/profile")
    public ApiResponse<UserProfileDTO> updateMyProfile(@RequestBody UpdateUserProfileRequest request) {
        return ApiResponse.ok(profileService.updateProfile(SecurityUtils.requireUser().id(), request));
    }

    /** 查看指定用户的扩展档案（公开信息） */
    @GetMapping("/{userId}/profile")
    public ApiResponse<UserProfileDTO> getUserProfile(@PathVariable Long userId) {
        return ApiResponse.ok(profileService.getProfile(userId));
    }
}
