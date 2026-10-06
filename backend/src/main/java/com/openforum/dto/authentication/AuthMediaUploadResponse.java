package com.openforum.dto.authentication;

public record AuthMediaUploadResponse(Long mediaId, String kind, String contentType, Long sizeBytes) {
}
