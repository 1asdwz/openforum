package com.openforum.controller;

import com.openforum.common.ApiResponse;
import com.openforum.dto.authentication.AuthChallengeDTO;
import com.openforum.dto.authentication.AuthMediaUploadResponse;
import com.openforum.dto.authentication.AuthTypeMetaDTO;
import com.openforum.dto.authentication.AuthenticationDTO;
import com.openforum.dto.authentication.AuthenticationSubmitRequest;
import com.openforum.dto.authentication.RetryRequest;
import com.openforum.security.AuthUser;
import com.openforum.security.SecurityUtils;
import com.openforum.service.AuthChallengeService;
import com.openforum.service.AuthMediaService;
import com.openforum.service.AuthenticationService;
import jakarta.validation.Valid;
import org.springframework.core.io.InputStreamResource;
import org.springframework.core.io.Resource;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RequestPart;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.multipart.MultipartFile;

import java.util.List;

@RestController
@RequestMapping("/authentications")
public class AuthenticationController {

    private final AuthenticationService authenticationService;
    private final AuthChallengeService challengeService;
    private final AuthMediaService mediaService;

    public AuthenticationController(AuthenticationService authenticationService,
                                    AuthChallengeService challengeService,
                                    AuthMediaService mediaService) {
        this.authenticationService = authenticationService;
        this.challengeService = challengeService;
        this.mediaService = mediaService;
    }

    @GetMapping("/types")
    public ApiResponse<List<AuthTypeMetaDTO>> types() {
        return ApiResponse.ok(authenticationService.types());
    }

    @GetMapping("/challenge")
    public ApiResponse<AuthChallengeDTO> challenge() {
        return ApiResponse.ok(challengeService.issue(SecurityUtils.requireUser().id()));
    }

    @PostMapping(value = "/media", consumes = MediaType.MULTIPART_FORM_DATA_VALUE)
    public ApiResponse<AuthMediaUploadResponse> uploadMedia(@RequestPart("file") MultipartFile file,
                                                            @RequestParam("kind") String kind) {
        return ApiResponse.ok(mediaService.upload(file, kind, SecurityUtils.requireUser().id()));
    }

    @GetMapping("/media/{mediaId}")
    public ResponseEntity<Resource> media(@PathVariable Long mediaId) {
        AuthUser user = SecurityUtils.requireUser();
        AuthMediaService.MediaResource media = mediaService.load(mediaId, user.id(), user.isAdmin());
        String contentType = media.contentType() == null ? "application/octet-stream" : media.contentType();
        return ResponseEntity.ok()
                .contentType(MediaType.parseMediaType(contentType))
                .contentLength(media.size())
                .header(HttpHeaders.CONTENT_DISPOSITION, "inline")
                .body(new InputStreamResource(media.stream()));
    }

    @PostMapping
    public ApiResponse<AuthenticationDTO> submit(@Valid @RequestBody AuthenticationSubmitRequest request) {
        return ApiResponse.ok(authenticationService.submit(request, SecurityUtils.requireUser().id()));
    }

    @GetMapping("/me")
    public ApiResponse<List<AuthenticationDTO>> my() {
        return ApiResponse.ok(authenticationService.my(SecurityUtils.requireUser().id()));
    }

    @GetMapping("/me/approved")
    public ApiResponse<List<String>> approved() {
        return ApiResponse.ok(authenticationService.approved(SecurityUtils.requireUser().id()));
    }

    @PostMapping("/{id}/retry")
    public ApiResponse<AuthenticationDTO> retry(@PathVariable Long id, @RequestBody RetryRequest request) {
        return ApiResponse.ok(authenticationService.retry(id, request, SecurityUtils.requireUser().id()));
    }
}
