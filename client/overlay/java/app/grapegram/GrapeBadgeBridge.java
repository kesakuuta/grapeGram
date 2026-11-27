package app.grapegram;

import android.content.Context;

import com.exteragram.messenger.api.dto.BadgeDTO;
import com.exteragram.messenger.api.model.ProfileStatus;
import com.exteragram.messenger.badges.source.ApiBadgeSource;
import com.exteragram.messenger.badges.source.BadgeInfo;
import com.exteragram.messenger.badges.BadgesController;

import org.telegram.messenger.MessagesController;
import org.telegram.messenger.NotificationCenter;
import org.telegram.messenger.Utilities;
import org.telegram.messenger.UserConfig;

import java.util.HashSet;
import java.util.Set;

/** Feeds the verified grapeGram list into the badge engine already present in exteraless. */
public final class GrapeBadgeBridge {

    private static final long GRAPEGRAM = 5449476181864779205L;
    private static final long FOUNDER = 5220045200780458122L;
    private static final long DEVELOPER = 5411229175971322671L;
    private static final long CONTRIBUTOR = 5393391313502609448L;

    /*
     * Track only entries owned by grapeGram. Never clear ApiBadgeSource wholesale:
     * other integrations may also have populated it.
     */
    private static final Set<Long> APPLIED_IDS = new HashSet<>();

    private GrapeBadgeBridge() {
    }

    public static void syncAsync(Context context) {
        Utilities.globalQueue.postRunnable(() -> sync(context.getApplicationContext()));
    }

    public static synchronized void sync(Context context) {
        // Refresh on launch; GrapeBadges falls back to the last verified cache offline.
        GrapeBadges.Result result = GrapeBadges.load(context, true);
        if (!result.ok) {
            return;
        }

        ApiBadgeSource source = BadgesController.INSTANCE.getApiBadgeSource();
        Set<Long> nextIds = new HashSet<>();

        for (GrapeBadges.Entry entry : result.entries) {
            if (entry == null || !entry.enabled) {
                continue;
            }

            final long id;
            try {
                id = Long.parseLong(entry.userId);
            } catch (NumberFormatException ignore) {
                continue;
            }
            if (id <= 0) {
                continue;
            }

            long documentId = documentId(entry.badge);
            source.put(id, new BadgeInfo(new BadgeDTO(documentId, "grapeGram"),
                    ProfileStatus.SUPPORTER, false));
            nextIds.add(id);
        }

        // Revoke stale entries previously installed by this bridge only.
        for (Long oldId : APPLIED_IDS) {
            if (!nextIds.contains(oldId)) {
                source.put(oldId, null);
            }
        }
        APPLIED_IDS.clear();
        APPLIED_IDS.addAll(nextIds);

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
