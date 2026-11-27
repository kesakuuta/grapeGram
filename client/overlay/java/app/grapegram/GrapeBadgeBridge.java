package app.grapegram;

import android.content.Context;

import com.exteragram.messenger.api.dto.BadgeDTO;
import com.exteragram.messenger.api.model.ProfileStatus;
import com.exteragram.messenger.badges.BadgesController;
import com.exteragram.messenger.badges.source.BadgeInfo;

import org.telegram.messenger.MessagesController;
import org.telegram.messenger.NotificationCenter;
import org.telegram.messenger.Utilities;
import org.telegram.messenger.UserConfig;

/** Feeds the verified grapeGram list into the badge engine already present in exteraless. */
public final class GrapeBadgeBridge {

    private static final long GRAPEGRAM = 5449476181864779205L;
    private static final long FOUNDER = 5220045200780458122L;
    private static final long DEVELOPER = 5411229175971322671L;
    private static final long CONTRIBUTOR = 5393391313502609448L;

    private GrapeBadgeBridge() {
    }

    public static void syncAsync(Context context) {
        Utilities.globalQueue.postRunnable(() -> sync(context.getApplicationContext()));
    }

    public static void sync(Context context) {
        GrapeBadges.Result result = GrapeBadges.load(context, false);
        if (!result.ok) {
            return;
        }
        for (GrapeBadges.Entry entry : result.entries) {
            long id;
            try {
                id = Long.parseLong(entry.userId);
            } catch (NumberFormatException ignore) {
                continue;
            }
            long documentId = documentId(entry.badge);
            BadgesController.INSTANCE.getApiBadgeSource().put(id,
                    new BadgeInfo(new BadgeDTO(documentId, "grapeGram"),
                            ProfileStatus.SUPPORTER, false));
        }
        NotificationCenter.getInstance(UserConfig.selectedAccount).postNotificationName(
                NotificationCenter.updateInterfaces, MessagesController.UPDATE_MASK_EMOJI_STATUS);
    }

    private static long documentId(String badge) {
        if ("founder".equals(badge)) {
            return FOUNDER;
        }
        if ("developer".equals(badge)) {
            return DEVELOPER;
        }
        if ("contributor".equals(badge)) {
            return CONTRIBUTOR;
        }
        return GRAPEGRAM;
    }
}
