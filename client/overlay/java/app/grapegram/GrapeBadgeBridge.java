package app.grapegram;

import android.content.Context;

import com.exteragram.messenger.api.dto.BadgeDTO;
import com.exteragram.messenger.api.model.ProfileStatus;
import com.exteragram.messenger.badges.BadgesController;
import com.exteragram.messenger.badges.source.ApiBadgeSource;
import com.exteragram.messenger.badges.source.BadgeInfo;

import org.telegram.messenger.MessagesController;
import org.telegram.messenger.NotificationCenter;
import org.telegram.messenger.Utilities;
import org.telegram.messenger.UserConfig;

import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;

/** Feeds the verified grapeGram list into the badge engine already present in exteraless. */
public final class GrapeBadgeBridge {

    private static final long GRAPEGRAM = 5449476181864779205L;
    private static final long FOUNDER = 5220045200780458122L;
    private static final long DEVELOPER = 5411229175971322671L;
    private static final long CONTRIBUTOR = 5393391313502609448L;

    /*
     * Keep ownership bookkeeping so a revoke restores a value that existed
     * before grapeGram instead of deleting another integration's badge.
     */
    private static final Map<Long, BadgeInfo> APPLIED = new HashMap<>();
    private static final Map<Long, BadgeInfo> PREVIOUS = new HashMap<>();

    private GrapeBadgeBridge() {
    }

    public static void syncAsync(Context context) {
        Utilities.globalQueue.postRunnable(() -> sync(context.getApplicationContext()));
    }

    public static synchronized void sync(Context context) {
        // Refresh on launch; GrapeBadges falls back to the last verified cache offline.
        sync(context, GrapeBadges.load(context, true));
    }

    /** Applies a result that has already been fetched by the caller, avoiding a second request. */
    public static synchronized void sync(Context context, GrapeBadges.Result result) {
        if (result == null || !result.ok) {
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
            BadgeInfo next = new BadgeInfo(new BadgeDTO(documentId, "grapeGram"),
                    ProfileStatus.SUPPORTER, false);

            if (!APPLIED.containsKey(id)) {
                PREVIOUS.put(id, source.getInfo(id));
            }
            source.put(id, next);
            APPLIED.put(id, next);
            nextIds.add(id);
        }

        // Revoke only our own current value; restore the previous provider value if any.
        for (Long oldId : new HashSet<>(APPLIED.keySet())) {
            if (!nextIds.contains(oldId)) {
                if (APPLIED.get(oldId).equals(source.getInfo(oldId))) {
                    source.put(oldId, PREVIOUS.get(oldId));
                }
                APPLIED.remove(oldId);
                PREVIOUS.remove(oldId);
            }
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
