package org.telegram.ui.Components;

import android.graphics.drawable.Drawable;
import android.view.View;

import org.telegram.messenger.AndroidUtilities;
import org.telegram.messenger.DialogObject;
import org.telegram.messenger.LocaleController;
import org.telegram.messenger.R;
import org.telegram.tgnet.TLObject;
import org.telegram.tgnet.TLRPC;
import org.telegram.ui.ActionBar.Theme;
import org.telegram.ui.Components.Premium.PremiumGradient;

import app.grapegram.GrapeBadgeDrawable;
import app.grapegram.GrapeBadges;

/** Telegram status drawable with an additional signed grapeGram badge when assigned. */
public class StatusBadgeComponent {

    private final View parentView;
    private final AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable statusDrawable;
    private Drawable verifiedDrawable;
    private Drawable currentDrawable;

    public StatusBadgeComponent(View parentView) {
        this(parentView, 18);
    }

    public StatusBadgeComponent(View parentView, int sizeDp) {
        this.parentView = parentView;
        statusDrawable = new AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable(parentView, AndroidUtilities.dp(sizeDp));
        currentDrawable = statusDrawable;
    }

    public Drawable updateDrawable(TLObject object, int colorFilter, boolean animated) {
        if (object instanceof TLRPC.User) {
            return updateDrawable((TLRPC.User) object, null, colorFilter, animated);
        } else if (object instanceof TLRPC.Chat) {
            return updateDrawable(null, (TLRPC.Chat) object, colorFilter, animated);
        }
        return updateDrawable(null, null, colorFilter, animated);
    }

    public Drawable updateDrawable(TLRPC.User user, TLRPC.Chat chat, int colorFilter, boolean animated) {
        boolean hasStatus = false;
        if (chat != null && chat.verified) {
            statusDrawable.set(verifiedDrawable = (verifiedDrawable == null ? new CombinedDrawable(Theme.dialogs_verifiedDrawable, Theme.dialogs_verifiedCheckDrawable) : verifiedDrawable), animated);
            statusDrawable.setColor(null);
            hasStatus = true;
        } else if (chat != null && DialogObject.getEmojiStatusDocumentId(chat.emoji_status) != 0) {
            statusDrawable.set(DialogObject.getEmojiStatusDocumentId(chat.emoji_status), animated);
            statusDrawable.setColor(colorFilter);
            hasStatus = true;
        } else if (user != null && user.verified) {
            statusDrawable.set(verifiedDrawable = (verifiedDrawable == null ? new CombinedDrawable(Theme.dialogs_verifiedDrawable, Theme.dialogs_verifiedCheckDrawable) : verifiedDrawable), animated);
            statusDrawable.setColor(null);
            hasStatus = true;
        } else if (user != null && DialogObject.getEmojiStatusDocumentId(user.emoji_status) != 0) {
            statusDrawable.set(DialogObject.getEmojiStatusDocumentId(user.emoji_status), animated);
            statusDrawable.setColor(colorFilter);
            hasStatus = true;
        } else if (user != null && user.premium) {
            statusDrawable.set(PremiumGradient.getInstance().premiumStarDrawableMini, animated);
            statusDrawable.setColor(colorFilter);
            hasStatus = true;
        } else {
            statusDrawable.set((Drawable) null, animated);
            statusDrawable.setColor(null);
        }

        GrapeBadges.Entry badge = user == null ? null
                : GrapeBadges.find(parentView.getContext(), String.valueOf(user.id));
        if (badge != null) {
            Drawable icon = parentView.getContext().getDrawable(R.drawable.grape_icon_original);
            currentDrawable = new GrapeBadgeDrawable(
                    hasStatus ? statusDrawable : null,
                    icon,
                    GrapeBadgeDrawable.colorForBadge(badge.badge,
                            Theme.isCurrentThemeDark() ? badge.colorDark : badge.colorLight),
                    AndroidUtilities.dp(18),
                    AndroidUtilities.dp(2),
                    LocaleController.isRTL,
                    (int) (badge.alpha * 255),
                    badge.particles,
                    badge.animation);
        } else {
            currentDrawable = hasStatus ? statusDrawable : null;
        }
        return currentDrawable;
    }

    public Drawable getDrawable() {
        return currentDrawable;
    }

    public void onAttachedToWindow() {
        statusDrawable.attach();
        if (currentDrawable instanceof GrapeBadgeDrawable) {
            ((GrapeBadgeDrawable) currentDrawable).setCallback((Drawable.Callback) parentView);
        }
    }

    public void onDetachedFromWindow() {
        if (currentDrawable instanceof GrapeBadgeDrawable) {
            ((GrapeBadgeDrawable) currentDrawable).setCallback(null);
        }
        statusDrawable.detach();
    }
}
