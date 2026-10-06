package com.openforum.plugin;

import com.openforum.entity.enums.AuthStatus;
import com.openforum.entity.enums.ReviewMethod;
import org.springframework.stereotype.Component;

import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;
import java.io.ByteArrayInputStream;
import java.util.ArrayList;
import java.util.List;

/**
 * 启发式证件审核：检查媒体存在性、可解码与清晰度下限。
 * 只能判 REJECTED（明显不合格）或 PENDING，绝不自动通过。
 */
@Component
public class HeuristicDocumentPlugin implements DocumentReviewPlugin {

    private static final int MIN_DIMENSION = 300;

    @Override
    public String getKey() {
        return "heuristic-document";
    }

    @Override
    public String getName() {
        return "证件启发式校验";
    }

    @Override
    public PluginType getType() {
        return PluginType.DOCUMENT_REVIEW;
    }

    @Override
    public boolean isEnabledByDefault() {
        return true;
    }

    @Override
    public String getDescription() {
        return "校验证件图片数量、可解码性与分辨率，明显不合格时驳回，否则转人工";
    }

    @Override
    public boolean isAvailable() {
        return true;
    }

    @Override
    public ReviewMethod getReviewMethod() {
        return ReviewMethod.RULE;
    }

    @Override
    public ReviewResult reviewDocuments(DocumentReviewContext ctx) {
        List<String> problems = new ArrayList<>();
        if (ctx.documents().isEmpty()) {
            problems.add("缺少证件图片");
        }
        for (MediaRef document : ctx.documents()) {
            if (document.bytes() == null || document.bytes().length == 0) {
                problems.add("证件图片内容为空");
                continue;
            }
            try {
                BufferedImage image = ImageIO.read(new ByteArrayInputStream(document.bytes()));
                if (image == null) {
                    problems.add("证件图片无法解码");
                } else if (image.getWidth() < MIN_DIMENSION || image.getHeight() < MIN_DIMENSION) {
                    problems.add("证件图片分辨率过低（需 ≥ " + MIN_DIMENSION + "px）");
                }
            } catch (Exception ex) {
                problems.add("证件图片无法解码");
            }
        }
        if (!problems.isEmpty()) {
            return new ReviewResult(AuthStatus.REJECTED, 0.6, String.join("；", problems));
        }
        return new ReviewResult(AuthStatus.PENDING, 0.3, "证件材料结构完整，转人工/其他插件复核");
    }
}
