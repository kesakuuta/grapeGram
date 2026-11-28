package app.grapegram;

import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.ColorFilter;
import android.graphics.Paint;
import android.graphics.PixelFormat;
import android.graphics.Rect;
import android.graphics.drawable.Drawable;
import android.os.SystemClock;

/** Small animated badge marker, composable with Telegram's existing status drawable. */
public final class GrapeBadgeDrawable extends Drawable implements Drawable.Callback {

    private final Drawable primary;
    private final Drawable icon;
    private final Paint background = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final Paint particlesPaint = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final Paint glowPaint = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final int size;
    private final int gap;
    private final int colorAlpha;
    private final boolean reverse;
    private final boolean particles;
    private final String animation;
    private int styleAlpha;
    private int viewAlpha = 255;

    private final Runnable frameRunnable = new Runnable() {
        @Override
        public void run() {
            invalidateSelf();
            if (getCallback() != null && isAnimated()) {
                scheduleSelf(this, SystemClock.uptimeMillis() + 72);
            }
        }
    };

    public GrapeBadgeDrawable(Drawable primary, Drawable icon, int badgeColor, int size, int gap) {
        this(primary, icon, badgeColor, size, gap, false, 255, false, "none");
    }

    public GrapeBadgeDrawable(Drawable primary, Drawable icon, int badgeColor, int size, int gap, boolean reverse) {
        this(primary, icon, badgeColor, size, gap, reverse, 255, false, "none");
    }

    public GrapeBadgeDrawable(Drawable primary, Drawable icon, int badgeColor, int size, int gap,
                              boolean reverse, int alpha, boolean particles, String animation) {
        this.primary = primary;
        this.icon = icon;
        this.size = Math.max(1, size);
        this.gap = Math.max(0, gap);
        this.reverse = reverse;
        this.styleAlpha = Math.max(0, Math.min(255, alpha));
        this.particles = particles;
        this.animation = animation == null ? "none" : animation;
        background.setColor(Color.rgb(Color.red(badgeColor), Color.green(badgeColor), Color.blue(badgeColor)));
        colorAlpha = Color.alpha(badgeColor);
        particlesPaint.setColor(Color.WHITE);
        glowPaint.setColor(Color.rgb(Color.red(badgeColor), Color.green(badgeColor), Color.blue(badgeColor)));
        if (primary != null) {
            primary.setCallback(this);
        }
        if (icon != null) {
            icon.setCallback(this);
        }
    }

    public Drawable getPrimaryDrawable() {
        return primary;
    }

    public static int colorForBadge(String badge) {
        boolean dark = org.telegram.ui.ActionBar.Theme.isCurrentThemeDark();
        if ("founder".equals(badge)) {
            return dark ? 0xFFF0C46A : 0xFFD99A20;
        }
        if ("developer".equals(badge)) {
            return dark ? 0xFFB18BE8 : 0xFF8757D9;
        }
        if ("contributor".equals(badge)) {
            return dark ? 0xFF63C7B6 : 0xFF259F91;
        }
        return dark ? 0xFFB18BE8 : 0xFF7852C8;
    }

    public static int colorForBadge(String badge, String override) {
        if (override != null) {
            try {
                return Color.parseColor(override);
            } catch (IllegalArgumentException ignored) {
                // Invalid signed style falls back to the badge's safe default.
            }
        }
        return colorForBadge(badge);
    }

    private boolean isAnimated() {
        return "pulse".equals(animation) || "sparkle".equals(animation) || "glow".equals(animation);
    }

    @Override
    public void setCallback(Callback callback) {
        unscheduleSelf(frameRunnable);
        super.setCallback(callback);
        if (callback != null && isAnimated()) {
            scheduleSelf(frameRunnable, SystemClock.uptimeMillis() + 72);
        }
    }

    @Override
    public void draw(Canvas canvas) {
        Rect bounds = getBounds();
        int left = bounds.left;
        int top = bounds.top;
        int combinedAlpha = styleAlpha * viewAlpha / 255;
        float phase = (SystemClock.uptimeMillis() % 1600L) / 1600.0f * 6.2831853f;
        float pulse = "pulse".equals(animation) ? 0.94f + 0.06f * (0.5f + 0.5f * (float) Math.sin(phase)) : 1.0f;

        int badgeLeft = primary == null ? left : left + (reverse ? 0 : size + gap);
        int primaryLeft = primary == null ? left : left + (reverse ? size + gap : 0);
        if (primary != null) {
            primary.setBounds(primaryLeft, top, primaryLeft + size, top + size);
            primary.setAlpha(combinedAlpha);
            primary.draw(canvas);
        }

        float cx = badgeLeft + size / 2.0f;
        float cy = top + size / 2.0f;
        float radius = size * 0.5f * pulse;
        if ("glow".equals(animation)) {
            int glowAlpha = combinedAlpha * (int) (50 + 35 * (0.5f + 0.5f * (float) Math.sin(phase))) / 255;
            glowPaint.setAlpha(glowAlpha * colorAlpha / 255);
            canvas.drawCircle(cx, cy, radius + size * 0.22f, glowPaint);
        }
        background.setAlpha(combinedAlpha * colorAlpha / 255);
        canvas.drawCircle(cx, cy, radius, background);

        if (icon != null) {
            int iconSize = Math.max(1, (int) (size * (0.72f * pulse)));
            int inset = (size - iconSize) / 2;
            icon.setBounds(badgeLeft + inset, top + inset, badgeLeft + inset + iconSize, top + inset + iconSize);
            icon.setAlpha(combinedAlpha * colorAlpha / 255);
            icon.draw(canvas);
        }

        if (particles) {
            int particleAlpha = combinedAlpha;
            if ("sparkle".equals(animation)) {
                particleAlpha = combinedAlpha * (int) (90 + 165 * (0.5f + 0.5f * (float) Math.sin(phase * 2.0f))) / 255;
            } else {
                particleAlpha = particleAlpha * 170 / 255;
            }
            particlesPaint.setAlpha(particleAlpha);
            float dot = Math.max(1.0f, size / 13.0f);
            canvas.drawCircle(cx - size * 0.31f, cy - size * 0.27f, dot, particlesPaint);
            canvas.drawCircle(cx + size * 0.31f, cy - size * 0.16f, dot * 0.75f, particlesPaint);
            canvas.drawCircle(cx + size * 0.12f, cy + size * 0.34f, dot * 0.65f, particlesPaint);
        }
    }

    @Override
    public int getIntrinsicWidth() {
        return size + (primary == null ? 0 : size + gap);
    }

    @Override
    public int getIntrinsicHeight() {
        return size;
    }

    @Override
    public void setAlpha(int value) {
        viewAlpha = Math.max(0, Math.min(255, value));
        invalidateSelf();
    }

    @Override
    public void setColorFilter(ColorFilter filter) {
        if (icon != null) {
            icon.setColorFilter(filter);
        }
        invalidateSelf();
    }

    @Override
    public int getOpacity() {
        return PixelFormat.TRANSLUCENT;
    }

    @Override
    public void invalidateDrawable(Drawable who) {
        invalidateSelf();
    }

    @Override
    public void scheduleDrawable(Drawable who, Runnable what, long when) {
        scheduleSelf(what, when);
    }

    @Override
    public void unscheduleDrawable(Drawable who, Runnable what) {
        unscheduleSelf(what);
    }
}
