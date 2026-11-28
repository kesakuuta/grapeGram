package app.grapegram;

import android.content.Context;
import android.content.SharedPreferences;

import com.exteragram.messenger.api.dto.BadgeDTO;
import com.exteragram.messenger.api.model.ProfileStatus;
import com.exteragram.messenger.badges.BadgesController;
import com.exteragram.messenger.badges.source.BadgeInfo;

import org.telegram.messenger.MessagesController;
import org.telegram.messenger.NotificationCenter;
import org.telegram.messenger.Utilities;
import org.telegram.messenger.UserConfig;

import java.util.Collections;
import java.util.HashSet;
import java.util.Set;

/** Feeds the verified grapeGram list into the badge engine already present in exteraless. */
public final class GrapeBadgeBridge {

    private static final long GRAPEGRAM = 5449476181864779205L;
    private static final long FOUNDER = 5220045200780458122L;
    private static final long DEVELOPER = 5411229175971322671L;
    private static final long CONTRIBUTOR = 5393391313502609448L;
    private static final String PREFS = "grape_badge_bridge";
    private static final String KEY_SYNCED_IDS = "synced_user_ids";
    private static final String KEY_LAST_REFRESH = "last_refresh_attempt";
    private static final long REFRESH_INTERVAL_MS = 60L * 60L * 1000L;
    private static volatile boolean refreshInProgress;

    private GrapeBadgeBridge() {
    }

    /** Refreshes the signed list on app startup, then applies the verified cache. */
    public static void syncAsync(Context context) {
        scheduleRefresh(context, true);
    }

    /** Refreshes after returning to foreground, at most once per hour. */
    public static void refreshIfDue(Context context) {
        scheduleRefresh(context, false);
    }

    private static void scheduleRefresh(Context context, boolean force) {
        if (context == null) {
            return;
        }
        final Context appContext = context.getApplicationContext();
        final SharedPreferences prefs = appContext.getSharedPreferences(PREFS, Context.MODE_PRIVATE);
        synchronized (GrapeBadgeBridge.class) {
            long now = System.currentTimeMillis();
            long lastAttempt = prefs.getLong(KEY_LAST_REFRESH, 0L);
            if (refreshInProgress || (!force && now - lastAttempt < REFRESH_INTERVAL_MS)) {
                return;
            }
            refreshInProgress = true;
            prefs.edit().putLong(KEY_LAST_REFRESH, now).apply();
        }
        Utilities.globalQueue.postRunnable(() -> {
            try {
                GrapeBadges.load(appContext, true);
                sync(appContext);
            } finally {
                synchronized (GrapeBadgeBridge.class) {
                    refreshInProgress = false;
                }
            }
        });
    }

    /** Applies an already refreshed cache without making a duplicate network request. */
    public static void syncCachedAsync(Context context) {
        final Context appContext = context.getApplicationContext();
        appContext.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
                .putLong(KEY_LAST_REFRESH, System.currentTimeMillis()).apply();
        Utilities.globalQueue.postRunnable(() -> sync(appContext));
    }

    public static synchronized void sync(Context context) {
        GrapeBadges.Result result = GrapeBadges.load(context, false);
        if (!result.ok) {
            return;
        }

        // Remove only entries previously installed by grapeGram. ApiBadgeSource is
        // shared with the host client, so clearing the entire source would erase
        // badges owned by other integrations.
        SharedPreferences prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE);
        Set<String> previousIds = new HashSet<>(
                prefs.getStringSet(KEY_SYNCED_IDS, Collections.<String>emptySet()));
        Set<String> nextIds = new HashSet<>();
        for (String previousId : previousIds) {
            try {
                long id = Long.parseLong(previousId);
                BadgeInfo existing = BadgesController.INSTANCE.getApiBadgeSource().getInfo(id);
                if (existing != null && isGrapeGramDocument(existing.getBadge().getDocumentId())) {
                    BadgesController.INSTANCE.getApiBadgeSource().put(id, null);
                }
            } catch (RuntimeException ignored) {
                // Ignore malformed IDs from an older local cache.
            }
        }

        for (GrapeBadges.Entry entry : result.entries) {
            if (!entry.enabled || !isKnownBadge(entry.badge)) {
                continue;
            }
            final long id;
            try {
                id = Long.parseLong(entry.userId);
                if (id <= 0) {
                    continue;
                }
            } catch (NumberFormatException ignored) {
                continue;
            }

            long documentId = documentId(entry.badge);
            ProfileStatus status = "developer".equals(entry.badge)
                    ? ProfileStatus.DEVELOPER : ProfileStatus.SUPPORTER;
            BadgesController.INSTANCE.getApiBadgeSource().put(id,
                    new BadgeInfo(new BadgeDTO(documentId, "grapeGram"), status, false));
            nextIds.add(Long.toString(id));
        }
        prefs.edit().putStringSet(KEY_SYNCED_IDS, nextIds).apply();

        NotificationCenter.getInstance(UserConfig.selectedAccount).postNotificationName(
                NotificationCenter.updateInterfaces, MessagesController.UPDATE_MASK_EMOJI_STATUS);
    }

    private static boolean isKnownBadge(String badge) {
        return "grapegram".equals(badge) || "founder".equals(badge)
                || "developer".equals(badge) || "contributor".equals(badge);
    }

    private static boolean isGrapeGramDocument(long documentId) {
        return documentId == GRAPEGRAM || documentId == FOUNDER
                || documentId == DEVELOPER || documentId == CONTRIBUTOR;
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
