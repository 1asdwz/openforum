package com.openforum.service;

import com.openforum.config.OpenForumAuthProperties;
import com.openforum.dto.media.MediaUploadResponse;
import com.openforum.entity.PostMedia;
import com.openforum.exception.ApiException;
import com.openforum.repository.PostMediaRepository;
import com.openforum.service.storage.StorageService;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.mock.web.MockMultipartFile;
import org.springframework.web.multipart.MultipartFile;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyLong;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class PostMediaServiceTest {

    private static final Long USER_ID = 1L;
    private static final long MAX_IMAGE_BYTES = 100L;
    private static final long MAX_VIDEO_BYTES = 200L;

    @Mock
    private PostMediaRepository mediaRepository;
    @Mock
    private StorageService storageService;

    private PostMediaService service;

    @BeforeEach
    void setUp() {
        OpenForumAuthProperties properties =
                new OpenForumAuthProperties(10, 5, MAX_IMAGE_BYTES, MAX_VIDEO_BYTES, 0.85);
        service = new PostMediaService(mediaRepository, storageService, properties);
    }

    private MockMultipartFile file(String contentType, byte[] content) {
        return new MockMultipartFile("file", "test.bin", contentType, content);
    }

    @Test
    void rejectsUnauthenticatedUser() {
        ApiException ex = assertThrows(ApiException.class,
                () -> service.upload(file("image/png", new byte[10]), null));
        assertEquals(401, ex.getCode());
    }

    @Test
    void rejectsEmptyFile() {
        ApiException ex = assertThrows(ApiException.class,
                () -> service.upload(file("image/png", new byte[0]), USER_ID));
        assertEquals(400, ex.getCode());
    }

    @Test
    void rejectsWhenStorageDisabled() {
        when(storageService.isEnabled()).thenReturn(false);
        ApiException ex = assertThrows(ApiException.class,
                () -> service.upload(file("image/png", new byte[10]), USER_ID));
        assertEquals(400, ex.getCode());
    }

    @Test
    void rejectsUnsupportedContentType() {
        when(storageService.isEnabled()).thenReturn(true);
        ApiException ex = assertThrows(ApiException.class,
                () -> service.upload(file("application/pdf", new byte[10]), USER_ID));
        assertEquals(400, ex.getCode());
        verify(storageService, never()).put(anyString(), any(), anyLong(), anyString());
    }

    @Test
    void rejectsImageExceedingLimit() {
        when(storageService.isEnabled()).thenReturn(true);
        byte[] content = new byte[(int) MAX_IMAGE_BYTES + 1];
        ApiException ex = assertThrows(ApiException.class,
                () -> service.upload(file("image/png", content), USER_ID));
        assertEquals(400, ex.getCode());
        verify(storageService, never()).put(anyString(), any(), anyLong(), anyString());
    }

    @Test
    void rejectsVideoExceedingLimit() {
        when(storageService.isEnabled()).thenReturn(true);
        byte[] content = new byte[(int) MAX_VIDEO_BYTES + 1];
        ApiException ex = assertThrows(ApiException.class,
                () -> service.upload(file("video/mp4", content), USER_ID));
        assertEquals(400, ex.getCode());
        verify(storageService, never()).put(anyString(), any(InputStream.class), anyLong(), anyString());
    }

    @Test
    void uploadsGifAndSavesMedia() {
        when(storageService.isEnabled()).thenReturn(true);
        when(mediaRepository.save(any(PostMedia.class))).thenAnswer(inv -> {
            PostMedia media = inv.getArgument(0);
            media.setId(7L);
            return media;
        });

        MediaUploadResponse response = service.upload(file("image/gif", new byte[10]), USER_ID);

        assertEquals(7L, response.id().longValue());
        assertEquals("image/gif", response.contentType());
        assertEquals(10L, response.sizeBytes().longValue());
        verify(storageService).put(anyString(), any(InputStream.class), eq(10L), eq("image/gif"));
        verify(mediaRepository).save(any(PostMedia.class));
    }

    @Test
    void loadReturnsContentTypeAndSize() {
        PostMedia media = new PostMedia();
        media.setId(3L);
        media.setUserId(USER_ID);
        media.setObjectKey("post/1/20261006/x.png");
        media.setContentType("image/png");
        media.setSizeBytes(123L);
        when(mediaRepository.findById(3L)).thenReturn(Optional.of(media));
        when(storageService.isEnabled()).thenReturn(true);
        when(storageService.get("post/1/20261006/x.png"))
                .thenReturn(new ByteArrayInputStream(new byte[123]));

        PostMediaService.MediaResource resource = service.load(3L);

        assertEquals("image/png", resource.contentType());
        assertEquals(123L, resource.size());
        assertNotNull(resource.stream());
    }

    @Test
    void loadMissingIdThrowsNotFound() {
        when(mediaRepository.findById(99L)).thenReturn(Optional.empty());
        ApiException ex = assertThrows(ApiException.class, () -> service.load(99L));
        assertEquals(404, ex.getCode());
    }
}
