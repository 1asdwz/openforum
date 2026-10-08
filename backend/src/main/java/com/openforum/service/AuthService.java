package com.openforum.service;

import com.openforum.config.OpenForumGoogleProperties;
import com.openforum.dto.auth.AuthResponse;
import com.openforum.dto.auth.LoginRequest;
import com.openforum.dto.auth.RegisterRequest;
import com.openforum.entity.User;
import com.openforum.entity.enums.UserRole;
import com.openforum.exception.ApiException;
import com.openforum.repository.UserRepository;
import com.openforum.security.GoogleTokenVerifier;
import com.openforum.security.JwtTokenProvider;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.UUID;

@Service
public class AuthService {

    private static final int MAX_USERNAME_LENGTH = 50;

    private final UserRepository userRepository;
    private final PasswordEncoder passwordEncoder;
    private final JwtTokenProvider tokenProvider;
    private final UserService userService;
    private final OpenForumGoogleProperties googleProperties;
    private final GoogleTokenVerifier googleTokenVerifier;

    public AuthService(UserRepository userRepository, PasswordEncoder passwordEncoder,
                       JwtTokenProvider tokenProvider, UserService userService,
                       OpenForumGoogleProperties googleProperties, GoogleTokenVerifier googleTokenVerifier) {
        this.userRepository = userRepository;
        this.passwordEncoder = passwordEncoder;
        this.tokenProvider = tokenProvider;
        this.userService = userService;
        this.googleProperties = googleProperties;
        this.googleTokenVerifier = googleTokenVerifier;
    }

    @Transactional
    public AuthResponse register(RegisterRequest request) {
        if (userRepository.existsByUsername(request.username())) {
            throw ApiException.badRequest("用户名已被占用");
        }
        if (request.email() != null && userRepository.existsByEmail(request.email())) {
            throw ApiException.badRequest("邮箱已被占用");
        }
        User user = new User();
        user.setUsername(request.username());
        user.setEmail(request.email());
        user.setPassword(passwordEncoder.encode(request.password()));
        user.setRole(UserRole.USER);
        user.setActive(true);
        userRepository.save(user);
        return new AuthResponse(tokenProvider.createToken(user), userService.me(user.getId()));
    }

    public AuthResponse login(LoginRequest request) {
        User user = userRepository
                .findByUsernameOrEmail(request.usernameOrEmail(), request.usernameOrEmail())
                .orElseThrow(() -> ApiException.unauthorized("账号或密码错误"));
        if (!user.isActive()) {
            throw ApiException.forbidden("账号已被禁用");
        }
        if (!passwordEncoder.matches(request.password(), user.getPassword())) {
            throw ApiException.unauthorized("账号或密码错误");
        }
        return new AuthResponse(tokenProvider.createToken(user), userService.me(user.getId()));
    }

    @Transactional
    public AuthResponse loginWithGoogle(String credential) {
        if (!googleProperties.isConfigured()) {
            throw ApiException.badRequest("Google 登录未配置");
        }
        GoogleTokenVerifier.GoogleUser googleUser = googleTokenVerifier.verify(credential);

        User user = userRepository.findByGoogleId(googleUser.googleId()).orElse(null);
        if (user == null && googleUser.email() != null && !googleUser.email().isBlank()) {
            user = userRepository.findByEmail(googleUser.email()).orElse(null);
            if (user != null) {
                user.setGoogleId(googleUser.googleId());
                userRepository.save(user);
            }
        }

        if (user != null) {
            if (!user.isActive()) {
                throw ApiException.forbidden("账号已被禁用");
            }
        } else {
            user = new User();
            user.setUsername(availableUsername(googleUser.email()));
            user.setEmail(googleUser.email());
            user.setPassword(passwordEncoder.encode(UUID.randomUUID().toString()));
            user.setAvatar(googleUser.picture());
            user.setGoogleId(googleUser.googleId());
            user.setRole(UserRole.USER);
            user.setActive(true);
            userRepository.save(user);
        }

        return new AuthResponse(tokenProvider.createToken(user), userService.me(user.getId()));
    }

    private String availableUsername(String email) {
        String raw = email == null ? "" : email;
        int at = raw.indexOf('@');
        if (at >= 0) {
            raw = raw.substring(0, at);
        }
        String base = raw.replaceAll("[^A-Za-z0-9_]", "");
        if (base.isEmpty()) {
            base = "user";
        }
        if (base.length() > MAX_USERNAME_LENGTH) {
            base = base.substring(0, MAX_USERNAME_LENGTH);
        }
        String candidate = base;
        int suffix = 0;
        while (userRepository.existsByUsername(candidate)) {
            suffix++;
            String suffixText = String.valueOf(suffix);
            int maxBaseLength = MAX_USERNAME_LENGTH - suffixText.length();
            String prefix = base.length() > maxBaseLength ? base.substring(0, maxBaseLength) : base;
            candidate = prefix + suffixText;
        }
        return candidate;
    }
}
