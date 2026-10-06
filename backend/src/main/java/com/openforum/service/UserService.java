package com.openforum.service;

import com.openforum.dto.user.UpdateProfileRequest;
import com.openforum.dto.user.UserDTO;
import com.openforum.common.PageResponse;
import com.openforum.entity.User;
import com.openforum.entity.enums.AuthStatus;
import com.openforum.exception.ApiException;
import com.openforum.repository.UserAuthenticationRepository;
import com.openforum.repository.UserRepository;
import org.springframework.data.domain.PageRequest;
import org.springframework.data.domain.Sort;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;

@Service
public class UserService {

    private final UserRepository userRepository;
    private final UserAuthenticationRepository authenticationRepository;

    public UserService(UserRepository userRepository, UserAuthenticationRepository authenticationRepository) {
        this.userRepository = userRepository;
        this.authenticationRepository = authenticationRepository;
    }

    public User require(Long id) {
        return userRepository.findById(id).orElseThrow(() -> ApiException.notFound("用户不存在"));
    }

    public UserDTO me(Long userId) {
        User user = require(userId);
        return toDto(user, approvedTypeNames(userId), true);
    }

    public UserDTO getPublicProfile(Long id) {
        User user = require(id);
        return toDto(user, approvedTypeNames(id), false);
    }

    @Transactional
    public UserDTO updateMe(Long userId, UpdateProfileRequest request) {
        User user = require(userId);
        if (request.avatar() != null) {
            user.setAvatar(request.avatar());
        }
        if (request.bio() != null) {
            user.setBio(request.bio());
        }
        userRepository.save(user);
        return toDto(user, approvedTypeNames(userId), true);
    }

    public PageResponse<UserDTO> listUsers(int page, int size) {
        var pageable = PageRequest.of(page, size, Sort.by(Sort.Direction.DESC, "createdAt"));
        return PageResponse.of(userRepository.findAll(pageable)
                .map(user -> toDto(user, approvedTypeNames(user.getId()), false)));
    }

    public List<String> approvedTypeNames(Long userId) {
        return authenticationRepository.findByUserIdAndStatus(userId, AuthStatus.APPROVED).stream()
                .map(auth -> auth.getType().name())
                .sorted()
                .toList();
    }

    public UserDTO toDto(User user, List<String> authentications, boolean includeEmail) {
        return new UserDTO(
                user.getId(),
                user.getUsername(),
                includeEmail ? user.getEmail() : null,
                user.getAvatar(),
                user.getBio(),
                user.getRole().name(),
                user.getCreatedAt(),
                authentications);
    }
}
