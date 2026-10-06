package com.openforum.repository;

import com.openforum.entity.SubForum;
import com.openforum.entity.enums.CustomPageStatus;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;

public interface SubForumRepository extends JpaRepository<SubForum, Long> {

    Optional<SubForum> findBySlug(String slug);

    boolean existsBySlug(String slug);

    Page<SubForum> findByNameContainingIgnoreCaseOrSlugContainingIgnoreCase(
            String name, String slug, Pageable pageable);

    List<SubForum> findByNameContainingIgnoreCaseOrSlugContainingIgnoreCase(String name, String slug);

    Page<SubForum> findByCustomPageStatus(CustomPageStatus status, Pageable pageable);

    List<SubForum> findByCustomPageStatusOrderByUpdatedAtDesc(CustomPageStatus status);
}
