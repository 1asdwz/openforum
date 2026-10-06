package com.openforum.controller;

import com.openforum.common.ApiResponse;
import com.openforum.dto.media.MediaUploadResponse;
import com.openforum.security.SecurityUtils;
import com.openforum.service.PostMediaService;
import org.springframework.core.io.InputStreamResource;
import org.springframework.core.io.Resource;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestPart;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.multipart.MultipartFile;

@RestController
@RequestMapping("/media")
public class MediaController {

    private final PostMediaService mediaService;

    public MediaController(PostMediaService mediaService) {
        this.mediaService = mediaService;
    }

    @PostMapping(consumes = "multipart/form-data")
    public ApiResponse<MediaUploadResponse> upload(@RequestPart("file") MultipartFile file) {
        return ApiResponse.ok(mediaService.upload(file, SecurityUtils.requireUser().id()));
    }

    @GetMapping("/{id}")
    public ResponseEntity<Resource> media(@PathVariable Long id) {
        PostMediaService.MediaResource media = mediaService.load(id);
        String contentType = media.contentType() == null ? "application/octet-stream" : media.contentType();
        return ResponseEntity.ok()
                .contentType(MediaType.parseMediaType(contentType))
                .contentLength(media.size())
                .header("Cache-Control", "public, max-age=31536000, immutable")
                .body(new InputStreamResource(media.stream()));
    }
}
