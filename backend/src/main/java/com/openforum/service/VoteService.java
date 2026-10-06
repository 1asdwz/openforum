package com.openforum.service;

import com.openforum.entity.Vote;
import com.openforum.entity.enums.VoteTargetType;
import com.openforum.repository.VoteRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.Optional;

@Service
public class VoteService {

    private final VoteRepository voteRepository;

    public VoteService(VoteRepository voteRepository) {
        this.voteRepository = voteRepository;
    }

    @Transactional
    public VoteOutcome apply(Long userId, VoteTargetType targetType, Long targetId, int value,
                             int currentUpvotes, int currentDownvotes) {
        int upvotes = currentUpvotes;
        int downvotes = currentDownvotes;
        Optional<Vote> existing = voteRepository.findByUserIdAndTargetTypeAndTargetId(userId, targetType, targetId);
        if (existing.isPresent()) {
            Vote vote = existing.get();
            int oldValue = vote.getValue();
            if (oldValue == 1) {
                upvotes--;
            } else if (oldValue == -1) {
                downvotes--;
            }
            if (value == 0) {
                voteRepository.delete(vote);
            } else {
                vote.setValue(value);
                voteRepository.save(vote);
            }
        } else if (value != 0) {
            Vote vote = new Vote();
            vote.setUserId(userId);
            vote.setTargetType(targetType);
            vote.setTargetId(targetId);
            vote.setValue(value);
            voteRepository.save(vote);
        }
        if (value == 1) {
            upvotes++;
        } else if (value == -1) {
            downvotes++;
        }
        upvotes = Math.max(0, upvotes);
        downvotes = Math.max(0, downvotes);
        return new VoteOutcome(upvotes, downvotes, upvotes - downvotes, value);
    }

    @Transactional
    public void clearVotes(VoteTargetType targetType, Long targetId) {
        voteRepository.deleteByTargetTypeAndTargetId(targetType, targetId);
    }

    public int myVote(Long userId, VoteTargetType targetType, Long targetId) {
        if (userId == null) {
            return 0;
        }
        return voteRepository.findByUserIdAndTargetTypeAndTargetId(userId, targetType, targetId)
                .map(Vote::getValue)
                .orElse(0);
    }

    public record VoteOutcome(int upvotes, int downvotes, int score, int myVote) {
    }
}
