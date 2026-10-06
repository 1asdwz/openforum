package com.openforum.service;

import com.openforum.entity.SubForum;
import com.openforum.entity.enums.ForumRole;
import com.openforum.repository.ForumMemberRepository;
import org.springframework.stereotype.Service;

import java.util.HashSet;
import java.util.Set;

@Service
public class ForumAccessService {

    private final ForumMemberRepository memberRepository;

    public ForumAccessService(ForumMemberRepository memberRepository) {
        this.memberRepository = memberRepository;
    }

    public boolean isMember(Long forumId, Long userId) {
        return userId != null && memberRepository.existsByForumIdAndUserId(forumId, userId);
    }

    public boolean canManage(SubForum forum, Long userId) {
        if (userId == null) {
            return false;
        }
        if (userId.equals(forum.getCreatorId())) {
            return true;
        }
        return memberRepository.findByForumIdAndUserId(forum.getId(), userId)
                .map(member -> member.getRole() == ForumRole.OWNER || member.getRole() == ForumRole.MODERATOR)
                .orElse(false);
    }

    public Set<Long> memberForumIds(Long userId) {
        if (userId == null) {
            return Set.of();
        }
        Set<Long> ids = new HashSet<>();
        memberRepository.findByUserIdOrderByJoinedAtDesc(userId).forEach(member -> ids.add(member.getForumId()));
        return ids;
    }
}
